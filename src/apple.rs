//! First Apple listening slice. UI state is separate from Spotify profile grants.
use std::time::Instant;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::player::{LocalState, LocalTrack, Playback};
use applifast_playback_probe::protocol::PlaybackItem;
pub mod models;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Read {
    Playlists,
    Albums,
    Artists,
    Album(String),
    AlbumTracks(String),
    Playlist(String),
    PlaylistTracks(String),
    Artist(String),
    ArtistAlbums(String),
    ArtistSongs(String),
    Search { serial: u64, library: bool },
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    #[serde(flatten)]
    pub item: PlaybackItem,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u32,
    pub catalog_id: Option<String>,
    #[serde(default)]
    pub artwork: Option<String>,
    #[serde(default)]
    pub album_id: Option<String>,
    #[serde(default)]
    pub artist_id: Option<String>,
}

impl Song {
    pub fn uri(&self) -> String {
        format!(
            "apple:track:{}.{}",
            match self.item.kind {
                applifast_playback_probe::protocol::ItemKind::Library => "library",
                applifast_playback_probe::protocol::ItemKind::Catalog => "catalog",
            },
            self.item.id
        )
    }
    pub fn track(&self) -> crate::api::models::Track {
        use crate::api::models::{Album, ArtistRef, Image, Track};
        let artists = vec![ArtistRef {
            name: self.artist.clone(),
            id: self.artist_id.clone(),
            uri: self
                .artist_id
                .as_ref()
                .map(|id| format!("apple:artist:{id}")),
        }];
        Track {
            id: Some(self.uri().trim_start_matches("apple:track:").to_owned()),
            uri: self.uri(),
            name: self.title.clone(),
            duration_ms: self.duration_ms,
            artists: artists.clone(),
            is_playable: Some(self.available()),
            album: Some(Album {
                id: self.album_id.clone().unwrap_or_default(),
                uri: self
                    .album_id
                    .as_ref()
                    .map_or_else(String::new, |id| format!("apple:album:{id}")),
                name: self.album.clone(),
                artists,
                images: self
                    .artwork
                    .iter()
                    .map(|url| Image {
                        url: url.clone(),
                        width: Some(640),
                        height: Some(640),
                    })
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
    pub fn available(&self) -> bool {
        self.item.play_params.is_some()
    }
    fn local_track(&self) -> LocalTrack {
        LocalTrack {
            uri: self.uri(),
            title: self.title.clone(),
            artists: self.track().artists,
            album: self.album.clone(),
            duration_ms: self.duration_ms,
            art_url: self.artwork.clone(),
            art_small_url: self.artwork.clone(),
            ..Default::default()
        }
    }
}

pub struct State {
    pub generation: u64,
    pub session: u64,
    pub authorized: bool,
    pub ready: bool,
    pub loading: bool,
    pub error: Option<String>,
    pub songs: Vec<Song>,
    pub next: Option<String>,
    pub token_path: String,
    pub filter: String,
    pub queue: Vec<Song>,
    pub known_songs: std::collections::HashMap<String, Song>,
    pub storefront: String,
    pub reads: std::collections::HashMap<u64, (Read, u32)>,
    pub next_reads: std::collections::HashMap<Read, (String, u32)>,
    pub pending_play: Option<crate::model::Action>,
    read_serial: u64,
    pub index: Option<usize>,
    pub local: LocalState,
    pending_index: Option<usize>,
    pending_playback: Option<Playback>,
    request_generation: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            generation: 1,
            session: 1,
            authorized: false,
            ready: false,
            loading: true,
            error: None,
            songs: Vec::new(),
            next: None,
            token_path: String::new(),
            filter: String::new(),
            queue: Vec::new(),
            known_songs: Default::default(),
            storefront: String::new(),
            reads: Default::default(),
            next_reads: Default::default(),
            pending_play: None,
            read_serial: 0,
            index: None,
            local: LocalState::default(),
            pending_index: None,
            pending_playback: None,
            request_generation: 0,
        }
    }
}

impl State {
    pub fn reset_host(&mut self) {
        self.generation += 1;
        self.session = 1;
        self.clear_account();
        self.ready = false;
        self.loading = true;
    }
    pub fn clear_account(&mut self) {
        self.authorized = false;
        self.loading = false;
        self.songs.clear();
        self.known_songs.clear();
        self.reads.clear();
        self.next_reads.clear();
        self.pending_play = None;
        self.queue.clear();
        self.next = None;
        self.index = None;
        self.pending_index = None;
        self.pending_playback = None;
        self.local = LocalState::default();
        self.error = None;
        self.request_generation = 0;
    }
    pub fn play(&mut self, index: usize) -> Option<Value> {
        let uris = self.songs.iter().map(Song::uri).collect::<Vec<_>>();
        self.play_uris(&uris, index)
    }
    pub fn play_uris(&mut self, uris: &[String], index: usize) -> Option<Value> {
        self.pending_play = None;
        let Some(selected) = uris.get(index).and_then(|uri| self.find_song(uri)) else {
            self.error=Some("This song has not been loaded from Apple Music. Open its collection and try again.".into());
            return None;
        };
        if !selected.available() {
            self.error = Some("Apple supplied no playback parameters for this song. Try it in Apple Music; cloud-only uploads are not guaranteed yet.".into());
            return None;
        }
        if uris.len() > 1000 {
            self.error = Some("This first listening slice supports contexts of at most 1,000 songs. Restart to reload the first page; larger queues arrive in the queue integration slice.".into());
            return None;
        }
        let queue: Option<Vec<_>> = uris
            .iter()
            .map(|uri| self.find_song(uri).cloned())
            .collect();
        let Some(queue) = queue else {
            self.error = Some("This song has not been loaded from Apple Music. Open its collection and try again.".into());
            return None;
        };
        self.queue = queue;
        let position = index;
        self.pending_index = Some(position);
        self.pending_playback = None;
        self.select(position);
        self.local.playback = Playback::Loading;
        self.local.position_ms = 0;
        self.error = None;
        self.request_generation += 1;
        Some(
            json!({"type":"play","items":self.queue.iter().map(|song| &song.item).collect::<Vec<_>>(),"index":position,"generation":self.request_generation}),
        )
    }
    pub fn find_song(&self, uri: &str) -> Option<&Song> {
        self.known_songs
            .get(uri)
            .or_else(|| self.songs.iter().find(|song| song.uri() == uri))
    }
    pub fn read(&mut self, target: Read, path: String, offset: u32) -> Value {
        self.read_serial += 1;
        self.reads.insert(self.read_serial, (target, offset));
        json!({"type":"request","id":self.read_serial,"path":path})
    }
    fn select(&mut self, index: usize) {
        if let Some(song) = self.queue.get(index) {
            if self.index != Some(index) {
                self.local.track_sequence += 1;
            }
            self.index = Some(index);
            self.local.track = Some(song.local_track());
        }
    }
    pub fn skip(&mut self, direction: i32) {
        if self.local.shuffle {
            self.pending_index = None;
            return;
        }
        let Some(index) = self.index else { return };
        let mut next = index as i64 + i64::from(direction);
        if self.local.repeat == crate::player::RepeatMode::Context && !self.queue.is_empty() {
            next = next.rem_euclid(self.queue.len() as i64);
        }
        if next >= 0 && (next as usize) < self.queue.len() {
            let paused = self.local.playback == Playback::Paused;
            self.pending_index = Some(next as usize);
            self.pending_playback = paused.then_some(Playback::Paused);
            self.select(next as usize);
            self.local.position_ms = 0;
            self.local.position_at = None;
            self.local.playback = if paused {
                Playback::Paused
            } else {
                Playback::Loading
            };
        }
    }
    pub fn intent(&mut self, command: Value) -> Value {
        self.request_generation += 1;
        json!({"type":"intent","generation":self.request_generation,"command":command})
    }
    pub fn toggle_play(&mut self) -> Value {
        let playing = self.local.playback == Playback::Playing;
        self.local.position_ms = self.local.position_now();
        self.local.playback = if playing {
            Playback::Paused
        } else {
            Playback::Playing
        };
        self.pending_playback = Some(self.local.playback);
        self.local.position_at = (!playing).then(Instant::now);
        json!({"type":if playing { "pause" } else { "resume" }})
    }
    pub fn seek(&mut self, ms: u32) {
        self.local.position_ms = ms;
        self.local.position_at = (self.local.playback == Playback::Playing).then(Instant::now);
        self.local.seek_sequence += 1;
    }
    /// Accept events only from this host and authorization lifetime.
    pub fn event(&mut self, generation: u64, event: &Value) -> Option<Value> {
        if generation != self.generation {
            return None;
        }
        if let Some(session) = event.get("session").and_then(Value::as_u64)
            && session != self.session
        {
            return None;
        }
        match event.get("type").and_then(Value::as_str) {
            Some("ready") => {
                self.storefront = event["storefront"]
                    .as_str()
                    .filter(|value| {
                        value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_lowercase())
                    })
                    .unwrap_or_default()
                    .to_owned();
                self.ready = true;
                self.loading = false;
                self.authorized = event["authorized"] == true;
                self.local.connected = self.authorized;
                if self.authorized {
                    self.loading = true;
                    return Some(json!({"type":"library","next":null}));
                }
            }
            Some("authorized") => {
                if let Some(storefront) = event["storefront"].as_str().filter(|value| {
                    value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_lowercase())
                }) {
                    self.storefront = storefront.to_owned();
                }
                self.authorized = true;
                self.local.connected = true;
                self.loading = true;
                return Some(json!({"type":"library","next":null}));
            }
            Some("signedOut") => self.clear_account(),
            Some("library") => {
                self.loading = false;
                let parsed: Result<Vec<Song>, _> = serde_json::from_value(event["items"].clone());
                match parsed {
                    Ok(songs) if songs.iter().all(|song| song.item.validate().is_ok()) => {
                        self.songs.extend(songs);
                        self.next = event["next"].as_str().map(str::to_owned);
                    }
                    _ => {
                        self.error = Some(
                            "Apple returned an invalid library page. Reload or sign in again."
                                .into(),
                        )
                    }
                }
            }
            Some("state") => {
                if event["requestGeneration"].as_u64() != Some(self.request_generation) {
                    return None;
                }
                let index = event["index"]
                    .as_u64()
                    .and_then(|index| usize::try_from(index).ok());
                if self.pending_index.is_some() && index != self.pending_index {
                    return None;
                }
                if let Some(index) = index {
                    self.select(index);
                }
                let playback = match event["status"].as_u64() {
                    Some(2) => Playback::Playing,
                    Some(3) => Playback::Paused,
                    Some(0) if index.is_some() => Playback::Paused,
                    Some(1 | 4 | 6 | 8) => Playback::Loading,
                    _ => Playback::Stopped,
                };
                if let Some(expected) = self.pending_playback {
                    if playback != expected {
                        return None;
                    }
                    self.pending_playback = None;
                }
                self.local.playback = playback;
                if matches!(self.local.playback, Playback::Playing | Playback::Paused) {
                    self.pending_index = None;
                }
                self.local.position_ms =
                    (event["position"].as_f64().unwrap_or(0.0).max(0.0) * 1000.0) as u32;
                self.local.position_at =
                    (self.local.playback == Playback::Playing).then(Instant::now);
            }
            Some("error") => {
                if event
                    .get("requestGeneration")
                    .and_then(Value::as_u64)
                    .is_some_and(|value| value != self.request_generation)
                {
                    return None;
                }
                self.loading = false;
                self.error = Some(
                    event["message"]
                        .as_str()
                        .unwrap_or(
                            "Apple playback failed. Check sign-in, subscription and connection.",
                        )
                        .into(),
                );
                self.pending_index = None;
                self.pending_playback = None;
                self.local.playback = Playback::Paused;
                self.local.position_at = None;
            }
            _ => {}
        }
        None
    }
    #[cfg(feature = "demo")]
    pub fn demo(saved: &[crate::api::models::SavedTrack]) -> Self {
        let mut state = Self {
            authorized: true,
            ready: true,
            loading: false,
            ..Default::default()
        };
        for (index, saved) in saved.iter().enumerate() {
            let id = format!("i.demo{index}");
            state.songs.push(Song {
                item: PlaybackItem {
                    kind: applifast_playback_probe::protocol::ItemKind::Library,
                    id: id.clone(),
                    play_params: Some(json!({"id":id,"kind":"song","isLibrary":true})),
                },
                title: saved.track.name.clone(),
                artist: saved.track.artist_names(),
                album: saved
                    .track
                    .album
                    .as_ref()
                    .map_or_else(String::new, |album| album.name.clone()),
                duration_ms: saved.track.duration_ms,
                catalog_id: None,
                artwork: saved.track.album.as_ref().and_then(|album| {
                    crate::api::models::pick_image(&album.images, 640).map(str::to_owned)
                }),
                album_id: saved.track.album.as_ref().map(|album| album.id.clone()),
                artist_id: saved
                    .track
                    .artists
                    .first()
                    .and_then(|artist| artist.id.clone()),
            });
        }
        state.local.volume = 32768;
        state.local.connected = true;
        state.local.repeat = crate::player::RepeatMode::Off;
        if !state.songs.is_empty() {
            state.play(0);
            state.local.playback = Playback::Playing;
            state.local.position_ms = 84000;
            state.local.position_at = None;
            state.pending_index = None;
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    #[ignore = "Uses the locally authorized Apple account and starts muted real playback"]
    fn native_host_restores_library_plays_seeks_and_pauses() {
        let (sender, events) = std::sync::mpsc::channel();
        let host = crate::player::AppleHost::start(None, move |event| {
            let _ = sender.send(event);
        });
        let result = || {
            let await_event = |kind: &str, matches: &dyn Fn(&Value) -> bool| -> Value {
                let deadline = Instant::now() + std::time::Duration::from_secs(45);
                loop {
                    let event = events
                        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                        .unwrap_or_else(|_| panic!("Playback host timed out awaiting {kind}"));
                    if event["type"] != "state" {
                        eprintln!("Host event: {}", event["type"]);
                    }
                    assert_ne!(event["type"], "error", "{}", event["message"]);
                    if event["type"] == kind && matches(&event) {
                        return event;
                    }
                }
            };
            let ready = await_event("ready", &|_| true);
            assert_eq!(
                ready["authorized"], true,
                "Authorize the local playback probe first"
            );
            host.send(json!({"type":"library","next":null}).to_string());
            let page = await_event("library", &|_| true);
            let songs: Vec<Song> = serde_json::from_value(page["items"].clone()).unwrap();
            let mut details = Vec::new();
            for (id,path) in [(10,"/v1/me/library/playlists?limit=10".to_owned()),(11,"/v1/me/library/albums?limit=10".to_owned()),(12,"/v1/me/library/artists?limit=10".to_owned()),(13,"/v1/me/library/search?term=music&types=library-songs,library-albums,library-artists,library-playlists&limit=10".to_owned()),(14,format!("/v1/catalog/{}/search?term=Bonobo&types=songs,albums,artists,playlists&limit=10",ready["storefront"].as_str().unwrap_or_default()))] {
                host.send(json!({"type":"request","id":id,"path":path}).to_string());
                let response=await_event("response",&|value| value["id"]==id);
                assert!(response["error"].is_null(),"{}",response["error"]);
                assert!(response["data"].is_object());
                if (10..=12).contains(&id) {
                    assert!(response["data"]["data"].is_array());
                    if let Some(resource)=response["data"]["data"].as_array().and_then(|rows|rows.first()) {
                        let kind=match id {10=>"playlists",11=>"albums",_=>"artists"};
                        let route=format!("/v1/me/library/{kind}/{}",resource["id"].as_str().unwrap());
                        details.push(route.clone());details.push(format!("{route}/{}?limit=100",if id==12 {"albums"}else{"tracks"}));
                    }
                } else {assert!(response["data"]["results"].is_object());}
                if id==14 {
                    let resource=response["data"]["results"]["artists"]["data"].as_array().and_then(|rows|rows.first()).expect("Catalog artist search returned no rows");
                    details.push(format!("/v1/catalog/{}/artists/{}/view/top-songs?limit=20",ready["storefront"].as_str().unwrap(),resource["id"].as_str().unwrap()));
                }
            }
            for (index, path) in details.iter().enumerate() {
                let id = 20 + index;
                host.send(json!({"type":"request","id":id,"path":path}).to_string());
                let response = await_event("response", &|value| value["id"] == id);
                assert!(
                    response["error"].is_null(),
                    "detail read {}: {}",
                    index,
                    response["error"]
                );
                assert!(response["data"]["data"].is_array());
            }
            let song = songs
                .iter()
                .find(|song| song.available())
                .expect("No playable library song");
            host.send(json!({"type":"volume","value":0}).to_string());
            host.send(
                json!({"type":"play","generation":1,"items":[song.item],"index":0}).to_string(),
            );
            await_event("state", &|event| {
                event["requestGeneration"] == 1
                    && event["status"] == 2
                    && event["position"].as_f64().unwrap_or(0.0) >= 1.0
            });
            host.send(
                json!({"type":"intent","generation":2,"command":{"type":"seek","seconds":30}})
                    .to_string(),
            );
            await_event("state", &|event| {
                event["requestGeneration"] == 2
                    && event["actualPosition"].as_f64().unwrap_or(0.0) >= 29.0
            });
            host.send(
                json!({"type":"intent","generation":3,"command":{"type":"pause"}}).to_string(),
            );
            await_event("state", &|event| {
                event["requestGeneration"] == 3 && event["status"] == 3
            });
            eprintln!(
                "Embedded Windows host: restored authorization; {} library rows; real library shelves, library/catalog search, playback, seek and pause passed.",
                songs.len()
            );
        };
        // Shut down even if an assertion fails, so the profile is never left locked.
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(result));
        host.shutdown();
        if let Err(failure) = outcome {
            std::panic::resume_unwind(failure);
        }
    }
    #[test]
    fn duplicate_occurrences_and_new_play_intent_survive_stale_and_failed_events() {
        let row = json!({"kind":"library","id":"i.matched","playParams":{"id":"i.matched","kind":"song","isLibrary":true,"catalogId":"123"},"catalogId":"123","title":"Library copy","artist":"Example","album":"Example","durationMs":180000});
        let mut state = State::default();
        state.event(
            1,
            &json!({"type":"library","session":1,"items":[row.clone(),row],"next":null}),
        );
        let command = state.play(1).unwrap();
        assert_eq!(command["index"], 1);
        assert_eq!(command["items"].as_array().unwrap().len(), 2);
        assert_eq!(command["items"][1]["id"], "i.matched");
        state.event(1, &json!({"type":"state","session":1,"requestGeneration":0,"index":0,"position":90,"status":2}));
        assert_eq!(state.index, Some(1));
        assert_eq!(state.local.position_ms, 0);
        state.local.playback = Playback::Playing;
        state.toggle_play();
        state.intent(json!({"type":"pause"}));
        state.event(1, &json!({"type":"state","session":1,"requestGeneration":1,"index":1,"position":90,"status":2}));
        assert_eq!(state.local.playback, Playback::Paused);
        state.event(1, &json!({"type":"state","session":1,"requestGeneration":2,"index":1,"position":0,"status":8}));
        assert_eq!(state.local.playback, Playback::Paused);
        state.seek(30000);
        state.intent(json!({"type":"seek","seconds":30}));
        state.event(1, &json!({"type":"state","session":1,"requestGeneration":2,"index":1,"position":90,"status":3}));
        assert_eq!(state.local.position_ms, 30000);
        state.event(
            1,
            &json!({"type":"error","session":1,"message":"Unavailable"}),
        );
        assert_eq!(state.queue.len(), 2);
        assert_eq!(state.index, Some(1));
    }
    #[test]
    fn pages_preserve_uploads_and_ignore_revoked_events() {
        let row = json!({"kind":"library","id":"i.upload","playParams":null,"catalogId":null,"title":"Upload","artist":"Me","album":"","durationMs":1234});
        let mut state = State::default();
        state.event(1, &json!({"type":"library","session":1,"items":[row.clone()],"next":"/v1/me/library/songs?offset=100"}));
        assert_eq!(state.songs.len(), 1);
        assert!(!state.songs[0].available());
        assert!(state.songs[0].catalog_id.is_none());
        assert!(state.play(0).is_none());
        state.session += 1;
        state.clear_account();
        state.event(
            1,
            &json!({"type":"library","session":1,"items":[row.clone()],"next":null}),
        );
        state.event(0, &json!({"type":"authorized","session":2}));
        assert!(state.songs.is_empty());
        assert!(!state.authorized);
        state.event(
            1,
            &json!({"type":"library","session":2,"items":[row],"next":null}),
        );
        assert_eq!(state.songs.len(), 1);
        assert!(state.next.is_none());
    }
}
