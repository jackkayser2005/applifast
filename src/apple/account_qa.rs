//! Opt-in account checks. Ordinary tests never send account writes.

use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use applifast_playback_probe::protocol::{Command, ItemKind};
use serde_json::{Value, json};

const QA_NAME: &str = "Applifast QA";

fn reserve_attempt(root: &Path) -> Result<(), &'static str> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("playlist-write-attempt"))
        .map_err(
            |_| "A playlist write may already have been attempted; do not retry automatically",
        )?;
    file.write_all(b"Create/append may have been issued. Inspect the receipt and Apple Music before any further attempt.\n")
        .and_then(|()| file.sync_all())
        .map_err(|_| "Cannot persist the write-attempt guard")
}

fn receipt(root: &Path, value: Value) -> Result<(), &'static str> {
    let temporary = root.join("playlist-receipt.tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| "Cannot create the receipt temporary file")?;
    let bytes = serde_json::to_vec_pretty(&value).map_err(|_| "Cannot encode the receipt")?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "Cannot persist the receipt")?;
    drop(file);
    crate::util::replace_file(&temporary, &root.join("playlist-receipt.json"))
        .map_err(|_| "Cannot replace the receipt atomically")
}

struct Host {
    host: Option<crate::player::AppleHost>,
    events: Receiver<Value>,
    serial: u64,
}

impl Host {
    fn start() -> Self {
        let (sender, events) = mpsc::channel();
        let host = crate::player::AppleHost::start(None, move |event| {
            let _ = sender.send(event);
        });
        Self {
            host: Some(host),
            events,
            serial: 12_000,
        }
    }

    fn wait(&self, kind: &str, id: Option<u64>) -> Result<Value, &'static str> {
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            let event = self
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| "Apple host did not confirm the operation")?;
            if event["type"] == "error" {
                return Err("Apple host reported an error");
            }
            if event["type"] == "ready" && event["authorized"] != true {
                return Err("Sign in locally before running the opt-in account check");
            }
            if event["type"] == kind && id.is_none_or(|id| event["id"] == id) {
                return Ok(event);
            }
        }
    }

    fn request(&mut self, make: impl FnOnce(u64) -> Command) -> Result<Value, &'static str> {
        self.serial += 1;
        let command = make(self.serial);
        self.send(&command)?;
        let response = self.wait("response", Some(self.serial))?;
        if !response["error"].is_null() {
            return Err(
                "Apple did not confirm the request; preserve the receipt and do not repeat writes",
            );
        }
        Ok(response["data"].clone())
    }

    fn send(&self, command: &Command) -> Result<(), &'static str> {
        command.validate().map_err(|_| "Invalid QA request")?;
        self.host
            .as_ref()
            .expect("host lives until Drop")
            .send(serde_json::to_string(command).map_err(|_| "Cannot encode the QA request")?);
        Ok(())
    }

    fn read(&mut self, path: String) -> Result<Value, &'static str> {
        self.request(|id| Command::Request { id, path })
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        if let Some(host) = self.host.take() {
            host.shutdown();
        }
    }
}

fn library_continuation(page: &Value, route: &str) -> Result<Option<String>, &'static str> {
    match page.get("next") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(path))
            if path.starts_with(&format!("{route}?"))
                && applifast_playback_probe::protocol::valid_read_path(path) =>
        {
            Ok(Some(path.clone()))
        }
        _ => Err("Unrecognized library continuation; no further request was sent"),
    }
}

#[test]
fn library_continuations_stop_on_invalid_or_cross_collection_routes() {
    let route = "/v1/me/library/songs";
    for page in [json!({}), json!({"next":null})] {
        assert_eq!(library_continuation(&page, route), Ok(None));
    }
    let next = "/v1/me/library/songs?offset=100&limit=100";
    assert_eq!(
        library_continuation(&json!({"next":next}), route),
        Ok(Some(next.into()))
    );
    for next in [
        json!(42),
        json!([]),
        json!(""),
        json!("https://api.music.apple.com/v1/me/library/songs?offset=100"),
        json!("/v1/me/library/albums?offset=100"),
        json!("/v1/me/library/songs?offset=100#fragment"),
        json!("/v1/me/library/songs/item?offset=100"),
    ] {
        assert!(library_continuation(&json!({"next":next}), route).is_err());
    }
}

#[test]
#[ignore = "Pages saved-account library songs/albums/artists/playlists through an isolated native host; no playback or account writes"]
fn native_pages_library_collections_without_playback() -> Result<(), &'static str> {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .canonicalize()
        .map_err(|_| "Missing workspace cache")?;
    let profile = std::env::var_os("LOCALAPPDATA").ok_or("Missing isolated profile")?;
    let profile = Path::new(&profile)
        .canonicalize()
        .map_err(|_| "Isolated profile must already exist")?;
    let root = cache.join("library-pagination-qa");
    if profile == root || !profile.starts_with(&root) {
        return Err("Use a fresh profile inside the workspace library-pagination-qa directory");
    }
    let mut host = Host::start();
    let ready = host.wait("ready", None)?;
    let mut expected_songs = HashMap::new();
    for (collection, resource_type) in [
        ("songs", "library-songs"),
        ("albums", "library-albums"),
        ("artists", "library-artists"),
        ("playlists", "library-playlists"),
    ] {
        let route = format!("/v1/me/library/{collection}");
        let mut next = Some(format!("{route}?limit=100"));
        let mut paths = HashSet::new();
        let mut identities = HashSet::new();
        let mut pages = 0;
        let mut without_catalog = 0;
        let mut without_play_params = 0;
        for _ in 0..200 {
            let Some(path) = next.take() else { break };
            if !paths.insert(path.clone()) {
                return Err("A library continuation repeated; no further request was sent");
            }
            let cursor_had_limit =
                reqwest::Url::parse(&format!("https://api.music.apple.com{path}"))
                    .map_err(|_| "Invalid library cursor")?
                    .query_pairs()
                    .any(|(key, _)| key == "limit");
            let page = host.read(path)?;
            let rows = page["data"].as_array().ok_or("Invalid library page")?;
            if pages < 2 {
                eprintln!(
                    "{}",
                    json!({"phase":"page-shape","collection":collection,
                    "page":pages+1,"rows":rows.len(),"incomingCursorHadLimit":cursor_had_limit})
                );
            }
            next = library_continuation(&page, &route)?;
            if rows.is_empty() && next.is_some() {
                return Err("An empty library page still has a continuation");
            }
            for row in rows {
                let id = row["id"]
                    .as_str()
                    .filter(|id| !id.is_empty())
                    .ok_or("Library resource has no identity")?;
                if row["type"] != resource_type || !identities.insert(id.to_owned()) {
                    return Err(
                        "Unexpected or repeated library identity; no consistent snapshot claim",
                    );
                }
                let expected = format!("library.{id}");
                let mapped = match collection {
                    "songs" => {
                        let song = super::models::song(row).ok_or("Library song did not map")?;
                        let params = row["attributes"]
                            .get("playParams")
                            .filter(|value| value.is_object());
                        if song.item.kind != ItemKind::Library
                            || song.item.id != id
                            || song.item.play_params.as_ref() != params
                            || song.catalog_id.as_deref()
                                != params.and_then(|value| value["catalogId"].as_str())
                        {
                            return Err(
                                "Library song identity or playback parameters changed during mapping",
                            );
                        }
                        without_catalog += usize::from(song.catalog_id.is_none());
                        without_play_params += usize::from(!song.available());
                        expected_songs.insert(id.to_owned(), song.item.play_params.clone());
                        let track = song.track();
                        if track.uri != format!("apple:track:{expected}")
                            || track.is_playable != Some(params.is_some())
                        {
                            return Err(
                                "Display mapping replaced a library song or its availability",
                            );
                        }
                        track.id.unwrap_or_default()
                    }
                    "albums" => super::models::album(row).id,
                    "artists" => super::models::artist(row).id,
                    _ => super::models::playlist(row).id,
                };
                if mapped != expected {
                    return Err("Display mapping changed a library resource identity");
                }
            }
            pages += 1;
        }
        if next.is_some() {
            return Err(
                "Library pagination exceeded the bounded QA cycle; collection not complete",
            );
        }
        eprintln!(
            "{}",
            json!({"collection":collection,"pages":pages,"rows":identities.len(),
            "songsWithoutCatalogId":without_catalog,"songsWithoutPlayParams":without_play_params,
            "identitiesAndParametersPreserved":true})
        );
    }
    let mut state = super::State::default();
    let mut command = state.event(1, &ready);
    let mut visited = HashSet::new();
    let mut pages = 0;
    for _ in 0..200 {
        let Some(value) = command.take() else { break };
        let typed: Command = serde_json::from_value(value.clone())
            .map_err(|_| "The production library command did not decode")?;
        if !matches!(typed, Command::Library { .. }) {
            return Err("The coordinator requested a non-library command; it was not sent");
        }
        host.send(&typed)?;
        let event = host.wait("library", value["id"].as_u64())?;
        if !event["error"].is_null() {
            return Err("The production Library command did not confirm");
        }
        let next = library_continuation(&event, "/v1/me/library/songs")?;
        if next
            .as_ref()
            .is_some_and(|path| !visited.insert(path.clone()))
        {
            return Err("The production Library continuation repeated");
        }
        command = state.event(1, &event);
        if state.library_error.is_some() || state.next != next {
            return Err("The production coordinator did not accept the library page");
        }
        if command.is_none() {
            command = next.map(|path| state.library_request(Some(path)));
        }
        pages += 1;
    }
    if command.is_some() || state.loading || state.songs.len() != expected_songs.len() {
        return Err("The production library did not finish the same collection as the raw reads");
    }
    let mut identities = HashSet::new();
    for song in &state.songs {
        if song.item.kind != ItemKind::Library
            || !identities.insert(song.item.id.clone())
            || expected_songs.get(&song.item.id) != Some(&song.item.play_params)
        {
            return Err("The production library changed an identity or its playback parameters");
        }
    }
    if !state.queue.is_empty() || state.index.is_some() {
        return Err("Read-only pagination changed the local playback queue");
    }
    eprintln!(
        "{}",
        json!({"collection":"production-songs","pages":pages,
        "rows":state.songs.len(),"identitiesAndParametersPreserved":true,"queueUntouched":true})
    );
    eprintln!(
        "Read-only library pagination completed; no playback, account write, popup or sign-out command sent."
    );
    Ok(())
}

#[test]
fn attempted_playlist_writes_are_exclusive_and_receipts_replace_atomically() {
    let root = std::env::temp_dir().join(format!("applifast-qa-receipt-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    reserve_attempt(&root).unwrap();
    assert!(reserve_attempt(&root).is_err());
    receipt(&root, json!({"phase":"first"})).unwrap();
    fs::write(root.join("playlist-receipt.tmp"), b"incomplete").unwrap();
    assert!(receipt(&root, json!({"phase":"second"})).is_err());
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(root.join("playlist-receipt.json")).unwrap())
            .unwrap()["phase"],
        "first"
    );
    fs::remove_file(root.join("playlist-receipt.tmp")).unwrap();
    receipt(&root, json!({"phase":"second"})).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(root.join("playlist-receipt.json")).unwrap())
            .unwrap()["phase"],
        "second"
    );
    for file in ["playlist-write-attempt", "playlist-receipt.json"] {
        fs::remove_file(root.join(file)).unwrap();
    }
    fs::remove_dir(root).unwrap();
}

#[test]
#[ignore = "Creates one Applifast QA playlist and appends an existing library song; requires explicit permission and an isolated profile/receipt directory"]
fn native_creates_and_appends_qa_playlist_once() -> Result<(), &'static str> {
    if std::env::var("APPLIFAST_ALLOW_QA_PLAYLIST_WRITE").as_deref() != Ok("create-one-qa-playlist")
    {
        return Err("Explicit local permission is required for the QA playlist write");
    }
    let root =
        std::env::var_os("APPLIFAST_QA_RECEIPT_DIR").ok_or("Missing QA receipt directory")?;
    let root = Path::new(&root)
        .canonicalize()
        .map_err(|_| "QA receipt directory must already exist")?;
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .canonicalize()
        .map_err(|_| "Missing workspace cache")?;
    let profile = std::env::var_os("LOCALAPPDATA").ok_or("Missing isolated profile")?;
    let profile = Path::new(&profile)
        .canonicalize()
        .map_err(|_| "Isolated profile must already exist")?;
    if root != cache.join("apple-account-qa")
        || !profile.starts_with(&root)
        || root.join("playlist-write-attempt").exists()
    {
        return Err(
            "Use a fresh isolated workspace profile and preserve every prior write attempt",
        );
    }
    let mut host = Host::start();
    host.wait("ready", None)?;
    let mut next = Some("/v1/me/library/playlists?limit=100".to_owned());
    for _ in 0..100 {
        let Some(path) = next.take() else { break };
        let page = host.read(path)?;
        let rows = page["data"].as_array().ok_or("Invalid playlist page")?;
        if rows.iter().any(|row| {
            row["type"] != "library-playlists" || row["attributes"]["name"].as_str().is_none()
        }) {
            return Err(
                "Playlist preflight contains an unrecognized resource; no writes were sent",
            );
        }
        if rows.iter().any(|row| row["attributes"]["name"] == QA_NAME) {
            return Err("An Applifast QA playlist already exists; no create or append was sent");
        }
        next = match page.get("next") {
            None | Some(Value::Null) => None,
            Some(Value::String(path)) => Some(path.clone()),
            _ => return Err("Invalid playlist continuation; no writes were sent"),
        };
    }
    if next.is_some() {
        return Err("Playlist preflight did not finish; no writes were sent");
    }
    let library = host.read("/v1/me/library/songs?limit=100".into())?;
    let songs: Vec<_> = library["data"]
        .as_array()
        .ok_or("Invalid song page")?
        .iter()
        .filter_map(super::models::song)
        .filter(|song| song.item.kind == ItemKind::Library)
        .take(2)
        .collect();
    if songs.len() != 2 {
        return Err("Two existing library songs are required before the QA write");
    }
    reserve_attempt(&root)?;
    receipt(
        &root,
        json!({"phase":"create-may-have-been-issued","name":QA_NAME}),
    )?;
    let created = host.request(|id| Command::CreatePlaylist {
        id,
        name: QA_NAME.into(),
        public: false,
        items: vec![songs[0].item.clone()],
    })?;
    let resource = created["data"]
        .as_array()
        .and_then(|rows| rows.first())
        .ok_or("Create response has no playlist; preserve the attempt receipt")?;
    let playlist = resource["id"]
        .as_str()
        .ok_or("Create response has no playlist ID")?
        .to_owned();
    if resource["type"] != "library-playlists" || resource["attributes"]["name"] != QA_NAME {
        return Err("Created playlist identity did not confirm; no append was sent");
    }
    receipt(
        &root,
        json!({"phase":"append-may-have-been-issued","playlist":playlist,"name":QA_NAME}),
    )?;
    host.request(|id| Command::AppendPlaylist {
        id,
        playlist: playlist.clone(),
        items: vec![songs[1].item.clone()],
    })?;
    for _ in 0..12 {
        let page = host.read(format!(
            "/v1/me/library/playlists/{playlist}/tracks?limit=100"
        ))?;
        if let Some(rows) = page["data"].as_array()
            && rows.len() == 2
            && rows
                .iter()
                .zip(&songs)
                .all(|(row, song)| row["type"] == "library-songs" && row["id"] == song.item.id)
        {
            receipt(
                &root,
                json!({"phase":"confirmed","playlist":playlist,"name":QA_NAME,"members":2}),
            )?;
            eprintln!(
                "Created one Applifast QA playlist and appended one existing library song; two exact library identities/order confirmed. No favorite or audio command sent."
            );
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err("Playlist membership has not confirmed yet; retain receipt and do not repeat either write")
}
