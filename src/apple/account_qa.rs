//! Opt-in account checks. Ordinary tests never send account writes.

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
                .map_err(|_| "Apple host did not confirm the operation; preserve the receipt")?;
            if event["type"] == "error" {
                return Err("Apple host reported an error; no automatic write retry is allowed");
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
        command.validate().map_err(|_| "Invalid QA request")?;
        self.host
            .as_ref()
            .expect("host lives until Drop")
            .send(serde_json::to_string(&command).map_err(|_| "Cannot encode the QA request")?);
        let response = self.wait("response", Some(self.serial))?;
        if !response["error"].is_null() {
            return Err(
                "Apple did not confirm the request; preserve the receipt and do not repeat writes",
            );
        }
        Ok(response["data"].clone())
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
