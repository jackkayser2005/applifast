//! Non-secret library metadata and local queue. Audio and tokens never go here.
use super::Song;
use applifast_playback_probe::protocol::QueueOrder;
use serde::{Deserialize, Serialize};
use std::{io::Read, path::Path};

const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u8,
    pub account_tag: String,
    pub storefront: String,
    pub songs: Vec<Song>,
    pub next: Option<String>,
    pub queue: Vec<Song>,
    pub order: QueueOrder,
    pub index: Option<usize>,
    pub position_ms: u32,
    pub shuffle: bool,
    pub repeat: u8,
}

pub fn valid_tag(tag: &str) -> bool {
    tag.len() == 64 && tag.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl Snapshot {
    pub fn valid_for(&self, tag: &str, storefront: &str) -> bool {
        self.version == 1
            && valid_tag(tag)
            && self.account_tag == tag
            && self.storefront == storefront
            && storefront.len() == 2
            && storefront.bytes().all(|byte| byte.is_ascii_lowercase())
            && self.songs.len() <= 100_000
            && self.queue.len() <= 1000
            && self.order.validate(self.queue.len(), self.index).is_ok()
            && self.repeat <= 2
            && self.songs.iter().chain(&self.queue).all(|song| {
                song.item.validate().is_ok()
                    && [&song.title, &song.artist, &song.album]
                        .iter()
                        .all(|text| text.len() <= 4096)
            })
            && self.next.as_ref().is_none_or(|path| {
                path.starts_with("/v1/me/library/songs?")
                    && applifast_playback_probe::protocol::valid_read_path(path)
            })
    }

    pub fn load(path: &Path, tag: &str, storefront: &str) -> Option<Self> {
        let file = std::fs::File::open(path).ok()?;
        if file.metadata().ok()?.len() > MAX_BYTES {
            return None;
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes).ok()?;
        if bytes.len() as u64 > MAX_BYTES {
            return None;
        }
        let snapshot: Self = serde_json::from_slice(&bytes).ok()?;
        snapshot.valid_for(tag, storefront).then_some(snapshot)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if !self.valid_for(&self.account_tag, &self.storefront) {
            return Err(std::io::Error::other("Invalid Apple session."));
        }
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(std::io::Error::other("Apple session exceeds 32 MiB."));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("json.tmp");
        let result = std::fs::write(&temporary, bytes)
            .and_then(|()| crate::util::replace_file(&temporary, path));
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }
}

pub fn clear(path: &Path) -> std::io::Result<()> {
    let mut result = Ok(());
    for file in [path.to_path_buf(), path.with_extension("json.tmp")] {
        if let Err(error) = std::fs::remove_file(file)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            result = Err(error);
        }
    }
    result
}
