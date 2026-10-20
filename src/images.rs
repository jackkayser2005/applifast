//! Album art: fetched once, kept on disk, decoded by egui on demand.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui::load::{Bytes, BytesLoadResult, BytesLoader, BytesPoll, ImagePoll, LoadError};
use sha1::{Digest, Sha1};

use crate::http::Http;

/// Maximum artwork bytes held in memory.
///
/// Time-based eviction does not work here: after creating a texture, egui no
/// longer requests its source bytes. Visible images were therefore evicted and
/// reloaded every two and a half minutes (#129).
///
/// Size-based eviction keeps visible images stable.
const HELD_BYTES: usize = 64 * 1024 * 1024;
const MAX_ART_BYTES: usize = 8 * 1024 * 1024;
const THUMBNAIL_FRAGMENT: &str = "#applifast-thumb-160";

/// A local cache identity, never an alternate URL sent to the artwork server.
pub(crate) fn thumbnail_uri(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or(url);
    (is_http(url) && !url.contains('#') && !path.to_ascii_lowercase().ends_with(".svg"))
        .then(|| format!("{url}{THUMBNAIL_FRAGMENT}"))
}

fn thumbnail_source(uri: &str) -> Option<&str> {
    let source = uri.strip_suffix(THUMBNAIL_FRAGMENT)?;
    thumbnail_uri(source).map(|_| source)
}

/// Decoded ColorImage plus the GPU texture, both RGBA.
fn decoded_and_texture_bytes(width: usize, height: usize) -> usize {
    2 * width.saturating_mul(height).saturating_mul(4)
}

/// What this loader answers for. egui offers it every URI, and artwork that
/// is not fetched over the network belongs to another loader.
fn is_http(uri: &str) -> bool {
    uri.starts_with("https://") || uri.starts_with("http://")
}

enum Entry {
    Pending,
    Ready {
        bytes: Option<Arc<[u8]>>,
        last_used: Instant,
        /// JPEG bytes still held, plus decoded image and texture once painted.
        retained: usize,
    },
    Failed(String),
}

struct Inner {
    entries: Mutex<HashMap<String, Entry>>,
    http: Http,
    runtime: tokio::runtime::Handle,
    cache_dir: PathBuf,
    thumbnail_workers: Arc<tokio::sync::Semaphore>,
}

#[derive(Clone)]
pub struct ArtLoader {
    inner: Arc<Inner>,
}

impl ArtLoader {
    pub fn new(http: impl Into<Http>, runtime: tokio::runtime::Handle, cache_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            inner: Arc::new(Inner {
                entries: Mutex::new(HashMap::new()),
                http: http.into(),
                runtime,
                cache_dir,
                thumbnail_workers: Arc::new(tokio::sync::Semaphore::new(2)),
            }),
        }
    }

    /// Bytes for `url`, from memory, disk, or the network.
    pub async fn fetch(&self, url: &str) -> Result<Arc<[u8]>, String> {
        self.inner.fetch(url).await
    }

    /// Whether artwork has finished loading from disk or the network.
    pub fn is_ready(&self, url: &str) -> bool {
        matches!(
            self.inner
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .get(url),
            Some(Entry::Ready { .. })
        )
    }

    /// Marks artwork as visible so size-based eviction keeps it stable.
    pub fn touch(&self, url: &str) {
        if let Some(Entry::Ready { last_used, .. }) = self
            .inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(url)
        {
            *last_used = Instant::now();
        }
    }

    /// Evicts failed entries and the oldest artwork above the memory limit.
    pub fn evict(&self, ctx: &egui::Context) {
        let letting_go: Vec<String> = {
            let entries = self.inner.entries.lock().unwrap_or_else(|p| p.into_inner());
            let mut failed: Vec<String> = Vec::new();
            let mut held: Vec<(String, Instant, usize)> = Vec::new();
            for (url, entry) in entries.iter() {
                match entry {
                    // Forget failures so a later request can retry.
                    Entry::Failed(_) => failed.push(url.clone()),
                    Entry::Ready {
                        last_used,
                        retained,
                        ..
                    } => {
                        held.push((url.clone(), *last_used, *retained));
                    }
                    Entry::Pending => {}
                }
            }
            failed.extend(over_budget(held, HELD_BYTES));
            failed
        };
        for url in letting_go {
            ctx.forget_image(&url);
            self.forget(&url);
        }
    }

    /// The disk-cache file holding `url`'s artwork, once it has been fetched.
    ///
    /// The cache is written atomically (a `.part` file, then a rename), so a
    /// file that is here at all holds a complete, successful response. The
    /// desktop media controls hand this path to the platform instead of the
    /// remote URL: macOS loads cover art itself, synchronously, inside a
    /// callback that cannot report a failure.
    pub fn cached_file(&self, url: &str) -> Option<PathBuf> {
        let path = self.inner.cache_path(url);
        std::fs::metadata(&path)
            .is_ok_and(|meta| meta.is_file() && meta.len() > 0)
            .then_some(path)
    }

    /// Starts the download for `url` while nothing is drawing it, so the
    /// media controls have a file to hand the platform, and answers whether
    /// this call is what started it.
    ///
    /// Artwork already held, already on its way, or addressed by a scheme
    /// this loader does not answer for is left alone.
    pub fn prefetch(&self, ctx: &egui::Context, url: &str) -> bool {
        if !is_http(url) {
            return false;
        }
        let mut entries = self.inner.entries.lock().unwrap_or_else(|p| p.into_inner());
        if entries.contains_key(url) {
            return false;
        }
        entries.insert(url.to_string(), Entry::Pending);
        drop(entries);
        self.inner.start(ctx, url.to_string());
        true
    }

    /// Drops held JPEG bytes once egui has made a texture. The disk cache
    /// remains for later reloads.
    pub fn release_bytes(&self, url: &str) {
        self.inner.drop_bytes(url);
    }

    /// Record decoded image + texture size after egui has uploaded the cover.
    pub fn note_decoded(&self, url: &str, width: usize, height: usize) {
        if let Some(Entry::Ready {
            bytes, retained, ..
        }) = self
            .inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(url)
        {
            let jpeg = bytes.as_ref().map(|bytes| bytes.len()).unwrap_or(0);
            *retained = jpeg + decoded_and_texture_bytes(width, height);
        }
    }

    pub fn clear_disk_cache(&self) -> std::io::Result<u64> {
        let mut removed = 0;
        for entry in std::fs::read_dir(&self.inner.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                removed += entry.metadata().map(|m| m.len()).unwrap_or(0);
                let _ = std::fs::remove_file(entry.path());
            }
        }
        Ok(removed)
    }
}

/// Which artwork to let go of so that what is kept fits `budget`,
/// oldest first.
///
/// "Oldest" is when egui last needed the bytes, which for a picture it
/// has already made a texture of is when it first loaded. That makes
/// this a rough order rather than a true reading of what is on screen,
/// which is why the budget is generous: being roughly right about which
/// to drop only matters once there is far more artwork than any window
/// is showing.
fn over_budget(mut held: Vec<(String, Instant, usize)>, budget: usize) -> Vec<String> {
    let mut total: usize = held.iter().map(|(_, _, bytes)| bytes).sum();
    if total <= budget {
        return Vec::new();
    }
    held.sort_by_key(|(_, last_used, _)| *last_used);
    let mut letting_go = Vec::new();
    for (url, _, bytes) in held {
        if total <= budget {
            break;
        }
        total = total.saturating_sub(bytes);
        letting_go.push(url);
    }
    letting_go
}

impl Inner {
    fn cache_path(&self, url: &str) -> PathBuf {
        let digest = Sha1::digest(url.as_bytes());
        let mut name = String::with_capacity(40);
        for byte in digest {
            use std::fmt::Write;
            let _ = write!(name, "{byte:02x}");
        }
        self.cache_dir.join(name)
    }

    async fn cached_bytes(&self, url: &str) -> Option<Arc<[u8]>> {
        if let Some(Entry::Ready {
            bytes: Some(bytes), ..
        }) = self
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(url)
        {
            return Some(Arc::clone(bytes));
        }
        let path = self.cache_path(url);
        tokio::task::spawn_blocking({
            let path = path.clone();
            move || std::fs::read(path).ok()
        })
        .await
        .ok()
        .flatten()
        .filter(|bytes| !bytes.is_empty() && bytes.len() <= MAX_ART_BYTES)
        .map(Arc::from)
    }

    async fn download(&self, url: &str) -> Result<Arc<[u8]>, String> {
        let response = self
            .http
            .client()?
            .get(url)
            .send()
            .await
            .map_err(|error| error.without_url().to_string())?;
        if !response.status().is_success() {
            return Err(format!("artwork request failed: {}", response.status()));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|error| error.without_url().to_string())?;
        if bytes.len() > MAX_ART_BYTES {
            return Err("artwork is too large".to_string());
        }
        Ok(Arc::from(bytes.as_ref()))
    }

    async fn fetch(self: &Arc<Self>, url: &str) -> Result<Arc<[u8]>, String> {
        if let Some(bytes) = self.cached_bytes(url).await {
            return Ok(bytes);
        }
        let bytes = if let Some(source) = thumbnail_source(url) {
            let permit = self
                .thumbnail_workers
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| "artwork worker stopped".to_string())?;
            let original = match self.cached_bytes(source).await {
                Some(bytes) => bytes,
                None => self.download(source).await?,
            };
            self.runtime
                .spawn_blocking(move || {
                    let _permit = permit;
                    thumbnail_bytes(&original)
                        .map(Arc::from)
                        .ok_or_else(|| "artwork thumbnail could not be decoded".to_string())
                })
                .await
                .map_err(|_| "artwork worker stopped".to_string())??
        } else {
            self.download(url).await?
        };
        let write_path = self.cache_path(url);
        let payload = Arc::clone(&bytes);
        self.runtime.spawn_blocking(move || {
            let temporary = write_path.with_extension("part");
            if std::fs::write(&temporary, &payload).is_ok() {
                let _ = std::fs::rename(&temporary, &write_path);
            }
        });
        Ok(bytes)
    }

    fn start(self: &Arc<Self>, ctx: &egui::Context, url: String) {
        let loader = Arc::clone(self);
        let ctx = ctx.clone();
        self.runtime.spawn(async move {
            let result = loader.fetch(&url).await;
            let entry = match result {
                Ok(bytes) => Entry::Ready {
                    retained: bytes.len(),
                    bytes: Some(bytes),
                    last_used: Instant::now(),
                },
                Err(error) => Entry::Failed(error),
            };
            loader
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(url, entry);
            ctx.request_repaint();
        });
    }

    fn drop_bytes(&self, url: &str) {
        if let Some(Entry::Ready {
            bytes, retained, ..
        }) = self
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(url)
            && let Some(held) = bytes.take()
        {
            *retained = retained.saturating_sub(held.len());
        }
    }
}

impl BytesLoader for ArtLoader {
    fn id(&self) -> &'static str {
        "spotifast::ArtLoader"
    }

    fn load(&self, ctx: &egui::Context, uri: &str) -> BytesLoadResult {
        if !is_http(uri) {
            return Err(LoadError::NotSupported);
        }
        let mut entries = self.inner.entries.lock().unwrap_or_else(|p| p.into_inner());
        match entries.get_mut(uri) {
            Some(Entry::Ready {
                bytes: Some(bytes),
                last_used,
                ..
            }) => {
                *last_used = Instant::now();
                Ok(BytesPoll::Ready {
                    size: None,
                    bytes: Bytes::Shared(Arc::clone(bytes)),
                    mime: None,
                })
            }
            Some(Entry::Ready {
                bytes: None,
                last_used,
                ..
            }) => {
                *last_used = Instant::now();
                entries.insert(uri.to_string(), Entry::Pending);
                drop(entries);
                self.inner.start(ctx, uri.to_string());
                Ok(BytesPoll::Pending { size: None })
            }
            Some(Entry::Pending) => Ok(BytesPoll::Pending { size: None }),
            Some(Entry::Failed(error)) => Err(LoadError::Loading(error.clone())),
            None => {
                entries.insert(uri.to_string(), Entry::Pending);
                drop(entries);
                self.inner.start(ctx, uri.to_string());
                Ok(BytesPoll::Pending { size: None })
            }
        }
    }

    fn forget(&self, uri: &str) {
        self.inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(uri);
    }

    fn forget_all(&self) {
        self.inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
    }

    fn byte_size(&self) -> usize {
        self.inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .values()
            .map(|entry| match entry {
                Entry::Ready {
                    bytes: Some(bytes), ..
                } => bytes.len(),
                _ => 0,
            })
            .sum()
    }
}

/// A colour that represents an album cover, suitable for tinting a dark or
/// light surface: the most common saturated hue, with its lightness pulled
/// into a range that still reads as a background.
pub fn accent_color(bytes: &[u8]) -> Option<[u8; 3]> {
    let decoded = image::load_from_memory(bytes).ok()?;
    let small = decoded.thumbnail(48, 48).to_rgb8();
    let mut buckets: HashMap<(u8, u8, u8), (u64, [u64; 3])> = HashMap::new();
    for pixel in small.pixels() {
        let [r, g, b] = pixel.0;
        let (max, min) = (r.max(g).max(b) as f32, r.min(g).min(b) as f32);
        let saturation = if max == 0.0 { 0.0 } else { (max - min) / max };
        let lightness = (max + min) / 510.0;
        // Weight toward vivid mid-tones so black borders and white text lose.
        let weight = (1.0 + saturation * 6.0) * (1.0 - (lightness - 0.5).abs() * 1.4).max(0.05);
        let weight = (weight * 100.0) as u64;
        let key = (r >> 4, g >> 4, b >> 4);
        let bucket = buckets.entry(key).or_insert((0, [0, 0, 0]));
        bucket.0 += weight;
        bucket.1[0] += r as u64 * weight;
        bucket.1[1] += g as u64 * weight;
        bucket.1[2] += b as u64 * weight;
    }
    let (_, (weight, sum)) = buckets.into_iter().max_by_key(|(_, (weight, _))| *weight)?;
    if weight == 0 {
        return None;
    }
    Some([
        (sum[0] / weight) as u8,
        (sum[1] / weight) as u8,
        (sum[2] / weight) as u8,
    ])
}

/// Blur applied to the full-window lyrics backdrop.
const LYRICS_BLUR: f32 = 9.0;

/// Blur applied to a library thumbnail standing in for a cover.
///
/// Softening happens at the same bounded size as the backdrop, so this is
/// weaker than the backdrop's radius: the thumbnail already carries little
/// detail, and a heavier blur smears it into flat colour.
const COVER_BLUR: f32 = 1.5;

/// Softened covers held at once, for library artwork seen recently.
const HELD_COVERS: usize = 64;

#[derive(Default)]
pub struct LyricsBackdrop {
    uri: Option<String>,
    requested: bool,
    retry_at: Option<Instant>,
    pending: Option<std::sync::mpsc::Receiver<Result<Option<egui::ColorImage>, String>>>,
    texture: Option<egui::TextureHandle>,
}

/// How long a backdrop whose artwork failed to download waits to ask again.
const BACKDROP_RETRY: Duration = Duration::from_secs(3);

impl LyricsBackdrop {
    /// The blurred backdrop for `uri`.
    ///
    /// The artwork is fetched here, from the disk cache or the network,
    /// rather than read from the bytes the image loader holds: a cover
    /// painted on the same screen releases those bytes as soon as its
    /// texture exists, and a backdrop waiting on them could wait forever.
    /// Until the new song's backdrop is ready the previous one stays, so a
    /// change of song never flashes black.
    pub fn texture(
        &mut self,
        ctx: &egui::Context,
        loader: &ArtLoader,
        uri: Option<&str>,
    ) -> Option<&egui::TextureHandle> {
        if self.uri.as_deref() != uri {
            *self = Self {
                uri: uri.map(str::to_owned),
                texture: self.texture.take().filter(|_| uri.is_some()),
                ..Default::default()
            };
        }
        let uri = uri?;
        let due = self.retry_at.is_none_or(|at| Instant::now() >= at);
        if !self.requested && due && is_http(uri) {
            self.requested = true;
            self.retry_at = None;
            let (tx, rx) = std::sync::mpsc::channel();
            self.pending = Some(rx);
            let ctx = ctx.clone();
            let inner = Arc::clone(&loader.inner);
            let url = uri.to_string();
            loader.inner.runtime.spawn(async move {
                let result = match inner.fetch(&url).await {
                    Ok(bytes) => inner
                        .runtime
                        .spawn_blocking(move || blurred_background(&bytes, LYRICS_BLUR))
                        .await
                        .map_err(|error| error.to_string()),
                    Err(error) => Err(error),
                };
                let _ = tx.send(result);
                ctx.request_repaint();
            });
        }
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(Ok(image)) => {
                    self.texture = image.map(|image| {
                        ctx.load_texture("lyrics-backdrop", image, egui::TextureOptions::LINEAR)
                    });
                    self.pending = None;
                }
                Ok(Err(_)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.requested = false;
                    self.retry_at = Some(Instant::now() + BACKDROP_RETRY);
                    ctx.request_repaint_after(BACKDROP_RETRY);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        self.texture.as_ref()
    }
}

fn terminal_art_error(error: &LoadError) -> bool {
    matches!(error, LoadError::NotSupported)
}

/// Softened artwork for covers that are still loading, one entry per URL.
///
/// A library row's small thumbnail is already on screen when its page opens,
/// so enlarging it blurred stands in for the cover until the full artwork
/// arrives. Blurring happens off the UI thread and the result is kept, so a
/// cover that is opened, left, and opened again shows softened art at once.
struct SoftenedCover {
    texture: egui::TextureHandle,
    last_used: Instant,
}

type SoftenedResult = (String, Option<egui::ColorImage>);

pub struct SoftenedCovers {
    textures: HashMap<String, SoftenedCover>,
    pending: HashSet<String>,
    failed: HashSet<String>,
    ready_tx: std::sync::mpsc::Sender<SoftenedResult>,
    ready_rx: std::sync::mpsc::Receiver<SoftenedResult>,
}

impl Default for SoftenedCovers {
    fn default() -> Self {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        Self {
            textures: HashMap::new(),
            pending: HashSet::new(),
            failed: HashSet::new(),
            ready_tx,
            ready_rx,
        }
    }
}

impl SoftenedCovers {
    fn receive_ready(&mut self, ctx: &egui::Context) {
        while let Ok((ready_uri, image)) = self.ready_rx.try_recv() {
            self.pending.remove(&ready_uri);
            if let Some(image) = image {
                if self.textures.len() >= HELD_COVERS
                    && let Some(oldest) = self
                        .textures
                        .iter()
                        .min_by_key(|(_, cover)| cover.last_used)
                        .map(|(uri, _)| uri.clone())
                {
                    self.textures.remove(&oldest);
                }
                let texture =
                    ctx.load_texture("softened-cover", image, egui::TextureOptions::LINEAR);
                self.textures.insert(
                    ready_uri,
                    SoftenedCover {
                        texture,
                        last_used: Instant::now(),
                    },
                );
            } else {
                self.failed.insert(ready_uri);
            }
        }
    }

    /// The softened cover for `uri`, blurring it in the background the first
    /// time it is asked for.
    pub fn texture(
        &mut self,
        ctx: &egui::Context,
        loader: &ArtLoader,
        uri: &str,
    ) -> Option<egui::TextureHandle> {
        self.receive_ready(ctx);
        if let Some(cover) = self.textures.get_mut(uri) {
            cover.last_used = Instant::now();
            return Some(cover.texture.clone());
        }
        if self.failed.contains(uri)
            || self.pending.contains(uri)
            || self.pending.len() >= HELD_COVERS
        {
            return None;
        }
        // The visible thumbnail has already been decoded by egui. Its source
        // bytes may have been released by paint_cover_url, which would discard
        // them again on the next frame before this consumer could use them.
        match ctx.try_load_image(uri, Default::default()) {
            Ok(ImagePoll::Ready { image }) => {
                let ready_tx = self.ready_tx.clone();
                let ready_uri = uri.to_string();
                let ctx = ctx.clone();
                self.pending.insert(ready_uri.clone());
                loader.inner.runtime.spawn_blocking(move || {
                    let image = softened_background(&image);
                    let _ = ready_tx.send((ready_uri, image));
                    ctx.request_repaint();
                });
            }
            Err(error) if terminal_art_error(&error) => {
                self.failed.insert(uri.to_string());
            }
            _ => {}
        }
        None
    }
}

fn softened_background(image: &egui::ColorImage) -> Option<egui::ColorImage> {
    let [width, height] = image.size;
    if width > 8192 || height > 8192 || image.pixels.len() > 64 * 1024 * 1024 / 4 {
        return None;
    }
    let pixels = image
        .pixels
        .iter()
        .flat_map(egui::Color32::to_srgba_unmultiplied)
        .collect();
    let image = image::RgbaImage::from_raw(width as u32, height as u32, pixels)?;
    Some(blurred_image(image.into(), COVER_BLUR))
}

fn blurred_background(bytes: &[u8], sigma: f32) -> Option<egui::ColorImage> {
    Some(blurred_image(decode_artwork(bytes)?, sigma))
}

fn thumbnail_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    let image = decode_artwork(bytes)?;
    let image = if image.width() > 160 || image.height() > 160 {
        image.thumbnail(160, 160)
    } else {
        image
    };
    let mut encoded = std::io::Cursor::new(Vec::new());
    image.write_to(&mut encoded, image::ImageFormat::Png).ok()?;
    Some(encoded.into_inner())
}

fn decode_artwork(bytes: &[u8]) -> Option<image::DynamicImage> {
    if bytes.len() > MAX_ART_BYTES {
        return None;
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().ok()
}

fn blurred_image(image: image::DynamicImage, sigma: f32) -> egui::ColorImage {
    let image = image.thumbnail(256, 256).blur(sigma).to_rgba8();
    egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    async fn serve_artwork(status: &str, bytes: Vec<u8>) -> (String, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/cover", listener.local_addr().unwrap());
        let header = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            bytes.len()
        );
        let server = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(10), async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    request.push(socket.read_u8().await.unwrap());
                    assert!(request.len() < 8192);
                }
                socket.write_all(header.as_bytes()).await.unwrap();
                socket.write_all(&bytes).await.unwrap();
            })
            .await
            .expect("the owned artwork request completes");
        });
        (url, server)
    }

    fn artwork_test_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    fn artwork_test_loader(runtime: &tokio::runtime::Runtime, dir: PathBuf) -> ArtLoader {
        ArtLoader::new(
            reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            runtime.handle().clone(),
            dir,
        )
    }

    async fn wait_for_artwork_file(path: &std::path::Path, expected: &[u8]) {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if tokio::fs::read(path)
                    .await
                    .is_ok_and(|bytes| bytes == expected)
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("the background writer completes while its runtime is alive");
    }

    #[test]
    fn thumbnail_identity_preserves_source_queries_and_skips_other_schemes_and_fragments() {
        let source = "https://example.com/custom/cover.png?version=1&size=original";
        let uri = thumbnail_uri(source).unwrap();
        assert_eq!(thumbnail_source(&uri), Some(source));
        for source in [
            "file:///cover.png",
            "https://example.com/cover#custom",
            "https://example.com/cover.SVG?v=1",
        ] {
            assert!(thumbnail_uri(source).is_none());
            assert!(thumbnail_source(source).is_none());
        }
    }

    #[test]
    fn local_thumbnail_reuses_original_disk_cache_without_upscaling_or_overwriting_it() {
        let dir = std::env::temp_dir().join(format!(
            "applifast-art-thumbnail-source-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(!dir.exists());
        let runtime = artwork_test_runtime();
        let loader = artwork_test_loader(&runtime, dir.clone());
        let source = "http://127.0.0.1:9/custom.png?original=1";
        cached_png(&loader, source, [10, 20, 30]);
        let original = std::fs::read(loader.inner.cache_path(source)).unwrap();
        let uri = thumbnail_uri(source).unwrap();
        runtime.block_on(async {
            let occupied = loader
                .inner
                .thumbnail_workers
                .clone()
                .acquire_many_owned(2)
                .await
                .unwrap();
            assert!(
                tokio::time::timeout(Duration::from_millis(50), loader.fetch(&uri))
                    .await
                    .is_err(),
                "a third thumbnail must wait before reading or decoding its original"
            );
            drop(occupied);
            let bytes = loader.fetch(&uri).await.unwrap();
            assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 64);
            wait_for_artwork_file(&loader.inner.cache_path(&uri), &bytes).await;
        });
        assert_eq!(
            std::fs::read(loader.inner.cache_path(source)).unwrap(),
            original
        );
        drop(loader);
        runtime.shutdown_timeout(Duration::from_secs(10));
        assert!(
            dir.canonicalize()
                .unwrap()
                .starts_with(std::env::temp_dir().canonicalize().unwrap())
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn fixed_artwork_thumbnail_is_small_and_reloads_without_network() {
        let dir = std::env::temp_dir().join(format!(
            "applifast-art-thumbnail-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(!dir.exists());
        let runtime = artwork_test_runtime();
        let loader = artwork_test_loader(&runtime, dir.clone());
        let mut encoded = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(640, 320, image::Rgba([200, 40, 40, 100]))
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        let (uri, expected) = runtime.block_on(async {
            let (source, server) = serve_artwork("200 OK", encoded.into_inner()).await;
            let uri = format!("{source}#applifast-thumb-160");
            let bytes = loader.fetch(&uri).await.unwrap();
            server.await.unwrap();
            let decoded = image::load_from_memory(&bytes).unwrap().to_rgba8();
            assert_eq!(decoded.dimensions(), (160, 80));
            assert_eq!(decoded.get_pixel(0, 0).0, [200, 40, 40, 100]);
            wait_for_artwork_file(&loader.inner.cache_path(&uri), &bytes).await;
            let ctx = egui::Context::default();
            egui_extras::install_image_loaders(&ctx);
            ctx.add_bytes_loader(Arc::new(loader.clone()));
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    match ctx
                        .try_load_texture(&uri, Default::default(), Default::default())
                        .unwrap()
                    {
                        egui::load::TexturePoll::Ready { texture } => {
                            assert_eq!(texture.size, egui::vec2(160.0, 80.0));
                            break;
                        }
                        egui::load::TexturePoll::Pending { .. } => {
                            tokio::time::sleep(Duration::from_millis(1)).await
                        }
                    }
                }
            })
            .await
            .expect("the normal egui loader uploads the small texture");
            (uri, bytes)
        });
        drop(loader);
        let loader = artwork_test_loader(&runtime, dir.clone());
        assert_eq!(runtime.block_on(loader.fetch(&uri)).unwrap(), expected);
        drop(loader);
        runtime.shutdown_timeout(Duration::from_secs(10));
        assert!(
            dir.canonicalize()
                .unwrap()
                .starts_with(std::env::temp_dir().canonicalize().unwrap())
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn downloaded_artwork_survives_caller_drop_and_reloads_without_network() {
        let dir =
            std::env::temp_dir().join(format!("spotifast-art-roundtrip-{}", std::process::id()));
        let runtime = artwork_test_runtime();
        let loader = artwork_test_loader(&runtime, dir.clone());
        let expected: Vec<u8> = (0..256 * 1024).map(|index| (index % 251) as u8).collect();
        let url = runtime.block_on(async {
            let (url, server) = serve_artwork("200 OK", expected.clone()).await;
            let bytes = loader.fetch(&url).await.expect("downloaded artwork");
            assert_eq!(&*bytes, expected.as_slice());
            drop(bytes);
            server.await.unwrap();
            wait_for_artwork_file(&loader.inner.cache_path(&url), &expected).await;
            url
        });
        let path = loader.inner.cache_path(&url);
        drop(loader);
        runtime.shutdown_timeout(Duration::from_secs(10));
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        assert!(!path.with_extension("part").exists());

        let runtime = artwork_test_runtime();
        let loader = artwork_test_loader(&runtime, dir.clone());
        // The server is gone. A new loader must recover the original bytes
        // from the completed cache, without requesting the URL again.
        assert_eq!(&*runtime.block_on(loader.fetch(&url)).unwrap(), expected);
        assert_eq!(loader.cached_file(&url), Some(path));
        drop(loader);
        runtime.shutdown_timeout(Duration::from_secs(10));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn artwork_cache_write_failure_keeps_download_usable() {
        let dir = std::env::temp_dir().join(format!(
            "spotifast-art-write-failure-{}",
            std::process::id()
        ));
        let runtime = artwork_test_runtime();
        let loader = artwork_test_loader(&runtime, dir.clone());
        let (bytes, path) = runtime.block_on(async {
            let (url, server) = serve_artwork("200 OK", b"complete artwork".to_vec()).await;
            let path = loader.inner.cache_path(&url);
            // A directory cannot be replaced by a file, even by an admin.
            std::fs::create_dir(&path).unwrap();
            let bytes = loader
                .fetch(&url)
                .await
                .expect("display does not depend on caching");
            server.await.unwrap();
            wait_for_artwork_file(&path.with_extension("part"), b"complete artwork").await;
            (bytes, path)
        });
        drop(loader);
        runtime.shutdown_timeout(Duration::from_secs(10));
        assert_eq!(&*bytes, b"complete artwork");
        assert!(
            path.is_dir(),
            "the failed replacement preserves the old path"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejected_artwork_responses_do_not_create_cache_files() {
        let dir =
            std::env::temp_dir().join(format!("spotifast-art-rejected-{}", std::process::id()));
        let runtime = artwork_test_runtime();
        let loader = artwork_test_loader(&runtime, dir.clone());
        runtime.block_on(async {
            for (status, body, message) in [
                ("404 Not Found", Vec::new(), "artwork request failed: 404"),
                ("200 OK", vec![1; MAX_ART_BYTES + 1], "artwork is too large"),
            ] {
                let (url, server) = serve_artwork(status, body).await;
                let error = loader.fetch(&url).await.unwrap_err();
                assert!(error.starts_with(message), "{error}");
                server.await.unwrap();
            }
        });
        drop(loader);
        runtime.shutdown_timeout(Duration::from_secs(10));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn transient_cover_load_errors_remain_retryable() {
        assert!(!terminal_art_error(&LoadError::Loading(
            "temporary network failure".into()
        )));
        assert!(!terminal_art_error(&LoadError::NoMatchingBytesLoader));
        assert!(terminal_art_error(&LoadError::NotSupported));
    }

    fn cached_png(loader: &ArtLoader, uri: &str, rgb: [u8; 3]) {
        let mut encoded = std::io::Cursor::new(Vec::new());
        image::RgbImage::from_pixel(64, 64, image::Rgb(rgb))
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        std::fs::write(loader.inner.cache_path(uri), encoded.into_inner()).unwrap();
    }

    /// Calls `texture` until `ready` accepts its answer, as frames would.
    fn backdrop_frames(
        runtime: &tokio::runtime::Runtime,
        backdrop: &mut LyricsBackdrop,
        ctx: &egui::Context,
        loader: &ArtLoader,
        uri: Option<&str>,
        mut each_frame: impl FnMut(),
        ready: impl Fn(Option<&egui::TextureHandle>) -> bool,
    ) {
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    each_frame();
                    if ready(backdrop.texture(ctx, loader, uri)) {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            })
            .await
            .expect("the backdrop settles");
        });
    }

    /// #561: a cover painted beside the backdrop releases the loader's bytes
    /// every frame once its texture exists. The backdrop must still appear.
    #[test]
    fn lyrics_backdrop_appears_while_a_cover_releases_the_same_artwork() {
        let runtime = artwork_test_runtime();
        let dir = std::env::temp_dir().join(format!("spotifast-backdrop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let loader = artwork_test_loader(&runtime, dir.clone());
        let ctx = egui::Context::default();
        let uri = "https://i.scdn.co/image/backdrop";
        cached_png(&loader, uri, [200, 40, 40]);
        loader.inner.entries.lock().unwrap().insert(
            uri.into(),
            Entry::Ready {
                retained: 0,
                bytes: None,
                last_used: Instant::now(),
            },
        );
        let mut backdrop = LyricsBackdrop::default();
        backdrop_frames(
            &runtime,
            &mut backdrop,
            &ctx,
            &loader,
            Some(uri),
            || loader.release_bytes(uri),
            |texture| texture.is_some(),
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A new song keeps the old backdrop until its own is ready, a failed
    /// download is asked for again, and no song clears the backdrop.
    #[test]
    fn lyrics_backdrop_holds_the_previous_song_and_retries_failures() {
        let runtime = artwork_test_runtime();
        let dir =
            std::env::temp_dir().join(format!("spotifast-backdrop-retry-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let loader = artwork_test_loader(&runtime, dir.clone());
        let ctx = egui::Context::default();
        let first = "https://i.scdn.co/image/first";
        // Nothing listens here, so the first attempt at this song fails.
        let second = "http://127.0.0.1:9/second";
        cached_png(&loader, first, [200, 40, 40]);
        let mut backdrop = LyricsBackdrop::default();
        backdrop_frames(
            &runtime,
            &mut backdrop,
            &ctx,
            &loader,
            Some(first),
            || {},
            |t| t.is_some(),
        );
        let shown = backdrop.texture.as_ref().unwrap().id();

        backdrop_frames(
            &runtime,
            &mut backdrop,
            &ctx,
            &loader,
            Some(second),
            || {},
            |t| {
                assert_eq!(t.map(|t| t.id()), Some(shown), "the old backdrop stays");
                true
            },
        );
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), async {
                while backdrop.retry_at.is_none() {
                    backdrop.texture(&ctx, &loader, Some(second));
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            })
            .await
            .expect("the failed download is noticed");
        });
        assert_eq!(backdrop.texture.as_ref().map(|t| t.id()), Some(shown));

        cached_png(&loader, second, [40, 40, 200]);
        backdrop.retry_at = Some(Instant::now());
        backdrop_frames(
            &runtime,
            &mut backdrop,
            &ctx,
            &loader,
            Some(second),
            || {},
            |t| t.is_some_and(|t| t.id() != shown),
        );

        assert!(backdrop.texture(&ctx, &loader, None).is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn blurred_background_rejects_oversized_decode_before_thumbnailing() {
        let image = image::RgbImage::from_pixel(8193, 1, image::Rgb([20, 30, 40]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        assert!(blurred_background(bytes.get_ref(), LYRICS_BLUR).is_none());
    }

    fn softened_test_image() -> egui::ColorImage {
        egui::ColorImage::filled([1, 1], egui::Color32::WHITE)
    }

    #[test]
    fn softened_cover_reuses_decoded_art_after_source_bytes_are_released() {
        let runtime = artwork_test_runtime();
        let dir =
            std::env::temp_dir().join(format!("spotifast-softened-decoded-{}", std::process::id()));
        let loader = artwork_test_loader(&runtime, dir.clone());
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        ctx.add_bytes_loader(Arc::new(loader.clone()));
        let uri = "https://i.scdn.co/image/decoded";
        let mut encoded = std::io::Cursor::new(Vec::new());
        image::RgbImage::from_pixel(64, 32, image::Rgb([20, 30, 40]))
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        let bytes: Arc<[u8]> = encoded.into_inner().into();
        loader.inner.entries.lock().unwrap().insert(
            uri.into(),
            Entry::Ready {
                retained: bytes.len(),
                bytes: Some(bytes),
                last_used: Instant::now(),
            },
        );
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), async {
                while !matches!(
                    ctx.try_load_texture(uri, Default::default(), Default::default()),
                    Ok(egui::load::TexturePoll::Ready { .. })
                ) {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                // Painting a visible cover releases its encoded bytes every
                // frame. There is deliberately no disk file or HTTP server.
                loader.release_bytes(uri);
                let mut covers = SoftenedCovers::default();
                loop {
                    loader.release_bytes(uri);
                    let texture = covers.texture(&ctx, &loader, uri);
                    assert!(
                        matches!(
                            loader.inner.entries.lock().unwrap().get(uri),
                            Some(Entry::Ready { bytes: None, .. })
                        ),
                        "softening must not restart the byte loader"
                    );
                    if let Some(texture) = texture {
                        assert_eq!(texture.size(), [256, 128]);
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                assert!(covers.pending.is_empty());
                assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
            })
            .await
            .expect("decoded artwork produces a softened cover");
        });
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn softened_cover_pending_requests_are_deduplicated() {
        let runtime = artwork_test_runtime();
        let dir =
            std::env::temp_dir().join(format!("spotifast-softened-pending-{}", std::process::id()));
        let loader = artwork_test_loader(&runtime, dir.clone());
        let ctx = egui::Context::default();
        let mut covers = SoftenedCovers::default();
        covers.pending.insert("pending".to_string());

        assert!(covers.texture(&ctx, &loader, "pending").is_none());
        assert_eq!(covers.pending.len(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn softened_cover_completion_and_failure_clear_pending_state() {
        let ctx = egui::Context::default();
        let mut covers = SoftenedCovers::default();
        covers.pending.insert("ready".to_string());
        covers.pending.insert("broken".to_string());
        covers
            .ready_tx
            .send(("ready".to_string(), Some(softened_test_image())))
            .unwrap();
        covers.ready_tx.send(("broken".to_string(), None)).unwrap();

        covers.receive_ready(&ctx);

        assert!(covers.pending.is_empty());
        assert!(covers.textures.contains_key("ready"));
        assert!(covers.failed.contains("broken"));
    }

    #[test]
    fn softened_cover_completion_evicts_the_oldest_texture() {
        let ctx = egui::Context::default();
        let mut covers = SoftenedCovers::default();
        let now = Instant::now();
        for index in 0..HELD_COVERS {
            covers.textures.insert(
                format!("cover-{index}"),
                SoftenedCover {
                    texture: ctx.load_texture(
                        format!("cover-{index}"),
                        softened_test_image(),
                        egui::TextureOptions::LINEAR,
                    ),
                    last_used: now + Duration::from_secs(index as u64),
                },
            );
        }
        covers.pending.insert("fresh".to_string());
        covers
            .ready_tx
            .send(("fresh".to_string(), Some(softened_test_image())))
            .unwrap();

        covers.receive_ready(&ctx);

        assert_eq!(covers.textures.len(), HELD_COVERS);
        assert!(!covers.textures.contains_key("cover-0"));
        assert!(covers.textures.contains_key("fresh"));
    }

    #[test]
    fn recreated_windows_release_artwork_and_reject_old_completions() {
        check_window_artwork_release(false);
    }

    #[test]
    fn closing_to_tray_releases_artwork_before_another_window_opens() {
        check_window_artwork_release(true);
    }

    fn check_window_artwork_release(close_before_reopen: bool) {
        let root = std::env::temp_dir().join(format!(
            "applifast-window-artwork-{}-{close_before_reopen}",
            std::process::id()
        ));
        let mut app = crate::app::App::new(
            &crate::backend::Waker::default(),
            crate::paths::AppDirs {
                config: root.join("config"),
                state: root.join("state"),
                cache: root.join("cache"),
            },
            crate::settings::Settings::default(),
            crate::app::AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
            },
        );
        let mut ctx = egui::Context::default();
        app.attach(&ctx);
        for mini in [true, false, true] {
            let old_manager = Arc::downgrade(&ctx.tex_manager());
            let (old_backdrop_tx, old_backdrop_rx) = std::sync::mpsc::channel();
            app.lyrics_backdrop = LyricsBackdrop {
                uri: Some("https://example.invalid/cover".into()),
                requested: true,
                pending: Some(old_backdrop_rx),
                texture: Some(ctx.load_texture(
                    "old-backdrop",
                    softened_test_image(),
                    egui::TextureOptions::LINEAR,
                )),
                ..Default::default()
            };
            let old_cover_tx = app.softened_covers.ready_tx.clone();
            old_cover_tx
                .send(("cover".into(), Some(softened_test_image())))
                .unwrap();
            app.softened_covers.receive_ready(&ctx);
            app.softened_covers.pending.insert("late".into());
            app.softened_covers.failed.insert("failed".into());

            ctx = egui::Context::default();
            if close_before_reopen {
                app.window_gone();
                assert!(app.window_hidden);
                assert!(
                    old_manager.upgrade().is_none(),
                    "closing to the tray must release artwork without waiting for reopen"
                );
                assert!(
                    old_backdrop_tx
                        .send(Ok(Some(softened_test_image())))
                        .is_err()
                );
                assert!(
                    old_cover_tx
                        .send(("late".into(), Some(softened_test_image())))
                        .is_err()
                );
            }
            app.settings.winamp_window = mini;
            app.attach(&ctx);

            assert!(
                old_manager.upgrade().is_none(),
                "recreating a window must release its artwork texture manager"
            );
            assert!(app.lyrics_backdrop.texture.is_none());
            assert!(app.lyrics_backdrop.uri.is_none());
            assert!(app.lyrics_backdrop.pending.is_none());
            assert!(!app.lyrics_backdrop.requested);
            assert!(app.softened_covers.textures.is_empty());
            assert!(app.softened_covers.pending.is_empty());
            assert!(app.softened_covers.failed.is_empty());
            assert!(
                old_backdrop_tx
                    .send(Ok(Some(softened_test_image())))
                    .is_err()
            );
            assert!(
                old_cover_tx
                    .send(("late".into(), Some(softened_test_image())))
                    .is_err()
            );

            app.softened_covers
                .ready_tx
                .send(("fresh".into(), Some(softened_test_image())))
                .unwrap();
            app.softened_covers.receive_ready(&ctx);
            assert_eq!(app.softened_covers.textures.len(), 1);
            let fresh = &app.softened_covers.textures["fresh"].texture;
            assert!(ctx.tex_manager().read().meta(fresh.id()).is_some());
        }
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn blurred_background_blurs_edges_and_bounds_texture_size() {
        let image = image::RgbImage::from_fn(640, 320, |x, _| {
            if x < 320 {
                image::Rgb([255, 0, 0])
            } else {
                image::Rgb([0, 0, 255])
            }
        });
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        let background = blurred_background(bytes.get_ref(), LYRICS_BLUR).expect("valid artwork");
        assert_eq!(background.size, [256, 128]);
        let center = background.pixels[64 * 256 + 128];
        assert!(
            center.r() > 40 && center.b() > 40,
            "the edge must be blurred: {center:?}"
        );
        assert!(
            background.pixels[0].r() > 240,
            "the cover's colors remain recognizable"
        );
        assert!(blurred_background(b"broken artwork", LYRICS_BLUR).is_none());
    }

    /// The media controls ask for a file rather than a URL, and have to be
    /// told "not yet" rather than handed a path to nothing: macOS loads cover
    /// art itself and dereferences a failed load without checking it, which
    /// takes the whole process with it.
    #[test]
    fn a_cached_file_is_named_only_once_it_is_really_there() {
        let dir = std::env::temp_dir().join(format!("spotifast-art-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime to hand the loader");
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let url = "https://i.scdn.co/image/abc";

        assert_eq!(loader.cached_file(url), None, "nothing downloaded yet");

        // A half-written download never appears under its real name -- the
        // cache renames one into place -- but an empty file is not artwork.
        let path = loader.inner.cache_path(url);
        std::fs::write(&path, b"").expect("an empty file");
        assert_eq!(loader.cached_file(url), None, "empty is not artwork");

        std::fs::write(&path, b"\xff\xd8\xff jpeg-ish").expect("a file with bytes");
        assert_eq!(loader.cached_file(url), Some(path));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn prefetching_starts_one_download_and_not_another() {
        let dir = std::env::temp_dir().join(format!("spotifast-prefetch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime to hand the loader");
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let ctx = egui::Context::default();
        let url = "https://i.scdn.co/image/never-drawn";

        assert!(loader.prefetch(&ctx, url), "nobody has asked for it yet");
        assert!(!loader.prefetch(&ctx, url), "it is already on its way");

        // A scheme the loader does not answer for is refused outright, the
        // same as in `load`, and nothing is remembered about it.
        let local = "file:///tmp/cover.jpg";
        assert!(
            !loader.prefetch(&ctx, local),
            "not a URL this loader fetches"
        );
        assert!(
            !loader
                .inner
                .entries
                .lock()
                .expect("the entries")
                .contains_key(local),
            "a URI it cannot fetch was remembered anyway"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn accent_color_finds_dominant_hue() {
        let mut image = image::RgbImage::new(16, 16);
        for (x, _, pixel) in image.enumerate_pixels_mut() {
            *pixel = if x < 12 {
                image::Rgb([20, 120, 200])
            } else {
                image::Rgb([255, 255, 255])
            };
        }
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        let color = accent_color(&bytes).unwrap();
        assert!(
            color[2] > color[0],
            "expected the blue field, got {color:?}"
        );
    }

    fn held(items: &[(&str, u64, usize)]) -> Vec<(String, Instant, usize)> {
        let base = Instant::now();
        items
            .iter()
            .map(|(url, age_secs, bytes)| {
                (
                    (*url).to_string(),
                    base - std::time::Duration::from_secs(*age_secs),
                    *bytes,
                )
            })
            .collect()
    }

    /// Rule: nothing is let go of while it all fits. This is the case
    /// that matters: an evening of listening never reaches the budget,
    /// so no cover ever blinks out and back (#129).
    #[test]
    fn artwork_that_fits_is_all_kept() {
        let art = held(&[("a", 600, 1000), ("b", 300, 1000), ("c", 1, 1000)]);
        assert!(over_budget(art, 10_000).is_empty());
    }

    /// Rule: over the budget, the oldest go first, and only as many as
    /// it takes to fit.
    #[test]
    fn the_oldest_go_until_the_rest_fit() {
        let art = held(&[
            ("oldest", 900, 1000),
            ("middle", 600, 1000),
            ("newest", 1, 1000),
        ]);
        assert_eq!(over_budget(art, 2000), vec!["oldest"]);
    }

    #[test]
    fn enough_go_to_get_under_the_budget() {
        let art = held(&[
            ("oldest", 900, 1000),
            ("middle", 600, 1000),
            ("newest", 1, 1000),
        ]);
        assert_eq!(over_budget(art, 900), vec!["oldest", "middle", "newest"]);
    }

    /// Rule: an empty gallery asks nothing of anyone.
    #[test]
    fn nothing_held_lets_nothing_go() {
        assert!(over_budget(Vec::new(), 0).is_empty());
    }

    fn retained_total(loader: &ArtLoader) -> usize {
        loader
            .inner
            .entries
            .lock()
            .expect("lock")
            .values()
            .map(|entry| match entry {
                Entry::Ready { retained, .. } => *retained,
                _ => 0,
            })
            .sum()
    }

    #[test]
    fn large_covers_evict_using_decoded_and_texture_sizes() {
        let one = decoded_and_texture_bytes(640, 640);
        assert_eq!(one, 2 * 640 * 640 * 4);
        let jpeg = 50_000usize;
        let dir = std::env::temp_dir().join(format!(
            "spotifast-art-budget-{}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("a runtime for eviction");
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let now = Instant::now();
        for i in 0..40 {
            let url = format!("https://i.scdn.co/image/{i}");
            loader.inner.entries.lock().expect("lock").insert(
                url,
                Entry::Ready {
                    bytes: Some(Arc::from(vec![0u8; jpeg])),
                    last_used: now - Duration::from_secs(40 - i),
                    retained: jpeg,
                },
            );
        }
        let jpeg_total = retained_total(&loader);
        assert!(
            jpeg_total < HELD_BYTES,
            "JPEG-only covers must still fit the budget: {jpeg_total}"
        );
        for i in 0..40 {
            let url = format!("https://i.scdn.co/image/{i}");
            loader.release_bytes(&url);
            loader.note_decoded(&url, 640, 640);
        }
        let before = retained_total(&loader);
        assert_eq!(before, 40 * one);
        assert!(
            before > HELD_BYTES,
            "decoded 640×640 covers plus textures must exceed 64 MiB: {before}"
        );
        let ctx = egui::Context::default();
        loader.evict(&ctx);
        let after = retained_total(&loader);
        assert!(
            after <= HELD_BYTES,
            "eviction must bring retained decoded+texture bytes under budget: after={after}"
        );
        assert!(
            after < before,
            "a long scroll of large covers must free memory: before={before} after={after}"
        );
        let entries = loader.inner.entries.lock().expect("lock");
        assert!(
            !entries.contains_key("https://i.scdn.co/image/0"),
            "the oldest scrolled-away cover must go first"
        );
        assert!(
            entries.contains_key("https://i.scdn.co/image/39"),
            "the cover just scrolled into view must stay"
        );
        drop(entries);
        loader.forget_all();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn releasing_bytes_reloads_from_disk_off_the_ui_thread() {
        use egui::load::BytesLoader;
        use std::time::Duration as StdDuration;

        let dir = std::env::temp_dir().join(format!(
            "spotifast-art-reload-{}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("a runtime for disk reload");
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let url = "https://i.scdn.co/image/reload";
        let path = loader.inner.cache_path(url);
        std::fs::create_dir_all(path.parent().expect("cache dir")).expect("cache dir");
        std::fs::write(&path, b"\xff\xd8\xff jpeg-ish").expect("cached jpeg");
        loader.inner.entries.lock().expect("lock").insert(
            url.to_string(),
            Entry::Ready {
                bytes: None,
                last_used: Instant::now(),
                retained: 0,
            },
        );
        let ctx = egui::Context::default();
        let first = loader.load(&ctx, url).expect("load");
        assert!(
            matches!(first, BytesPoll::Pending { .. }),
            "disk reload must not block the UI thread"
        );
        let deadline = Instant::now() + StdDuration::from_secs(2);
        loop {
            std::thread::sleep(StdDuration::from_millis(20));
            match loader.load(&ctx, url) {
                Ok(BytesPoll::Ready { .. }) => break,
                Ok(BytesPoll::Pending { .. }) if Instant::now() < deadline => continue,
                _ => panic!("reload did not finish"),
            }
        }
        loader.forget_all();
        assert!(loader.inner.entries.lock().expect("lock").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
