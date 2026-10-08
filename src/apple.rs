//! First Apple listening slice. UI state is separate from Spotify profile grants.
use std::time::Instant;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::player::{LocalState, LocalTrack, Playback};
use applifast_playback_probe::protocol::{PlaybackItem, QueueOrder};
use rand::seq::SliceRandom;
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
    pub order: QueueOrder,
    recent_adds: std::collections::HashMap<String, Instant>,
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
            order: QueueOrder::default(),
            recent_adds: Default::default(),
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
        self.order = QueueOrder::default();
        self.recent_adds.clear();
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
        if uris.len() + self.order.manual_count > 1000 {
            self.error = Some("This listening slice supports at most 1,000 queue occurrences. Clear manual additions or choose a smaller context.".into());
            return None;
        }
        let queue: Option<Vec<_>> = uris
            .iter()
            .map(|uri| self.find_song(uri).cloned())
            .collect();
        let Some(mut queue) = queue else {
            self.error = Some("This song has not been loaded from Apple Music. Open its collection and try again.".into());
            return None;
        };
        let manual = self.order.upcoming[..self.order.manual_count]
            .iter()
            .filter_map(|index| self.queue.get(*index).cloned())
            .collect::<Vec<_>>();
        let context_len = queue.len();
        queue.extend(manual);
        self.queue = queue;
        self.order = QueueOrder {
            upcoming: (context_len..self.queue.len()).collect(),
            manual_count: self.queue.len() - context_len,
            context: (0..context_len).collect(),
            history: Vec::new(),
        };
        let mut rest = if self.local.shuffle {
            (0..context_len)
                .filter(|at| *at != index)
                .collect::<Vec<_>>()
        } else {
            (index + 1..context_len).collect()
        };
        if self.local.shuffle {
            rest.shuffle(&mut rand::rng());
            self.order.context.shuffle(&mut rand::rng());
        }
        self.order.upcoming.extend(rest);
        let position = index;
        self.pending_index = Some(position);
        self.pending_playback = None;
        self.select(position);
        self.local.playback = Playback::Loading;
        self.local.position_ms = 0;
        self.error = None;
        self.request_generation += 1;
        Some(
            json!({"type":"play","items":self.queue.iter().map(|song| &song.item).collect::<Vec<_>>(),"index":position,"generation":self.request_generation,"order":self.order}),
        )
    }
    pub fn queue_command(&self) -> Value {
        json!({"type":"queue","items":self.queue.iter().map(|song| &song.item).collect::<Vec<_>>(),"index":self.index,"order":self.order})
    }
    pub fn selection_command(&mut self, playing: bool) -> Value {
        self.local.playback = if playing {
            Playback::Loading
        } else {
            Playback::Paused
        };
        self.pending_playback = (!playing).then_some(Playback::Paused);
        json!({"type":"select","index":self.index,"order":self.order,"playing":playing})
    }
    pub fn add_uris(
        &mut self,
        uris: &[String],
        position: usize,
        debounce: std::time::Duration,
        album: bool,
    ) -> Option<usize> {
        self.recent_adds.retain(|_, at| at.elapsed() < debounce);
        // Decide before inserting: repeats within this batch are distinct occurrences.
        let additions = uris
            .iter()
            .filter(|uri| album || !self.recent_adds.contains_key(*uri))
            .map(|uri| self.find_song(uri).cloned())
            .collect::<Option<Vec<_>>>();
        let Some(additions) = additions else {
            self.error = Some(
                "Open the song's collection and wait for it to load before adding it to queue."
                    .into(),
            );
            return None;
        };
        let additions = additions
            .into_iter()
            .filter(Song::available)
            .collect::<Vec<_>>();
        if additions.is_empty() {
            if !uris.iter().all(|uri| self.recent_adds.contains_key(uri)) {
                self.error = Some("Apple supplied no playable songs to add to queue.".into());
            }
            return None;
        }
        // ponytail: bounded wire snapshots allow 1,000 occurrences; paged queues are a later slice.
        let retained = self.retained_occurrences();
        if retained.len() + additions.len() > 1000 {
            self.error = Some("This listening slice supports at most 1,000 queue occurrences. Choose a smaller context.".into());
            return None;
        }
        self.compact_queue(&retained);
        let count = additions.len();
        let indices = self.queue.len()..self.queue.len() + count;
        for song in &additions {
            self.recent_adds.insert(song.uri(), Instant::now());
        }
        self.queue.extend(additions);
        let position = position.min(self.order.manual_count);
        self.order.upcoming.splice(position..position, indices);
        self.order.manual_count += count;
        self.error = None;
        Some(count)
    }
    pub fn clear_manual(&mut self) {
        self.order.upcoming.drain(..self.order.manual_count);
        self.order.manual_count = 0;
        self.recent_adds.clear();
        self.compact_queue(&self.retained_occurrences());
    }
    fn retained_occurrences(&self) -> std::collections::HashSet<usize> {
        self.order
            .context
            .iter()
            .chain(&self.order.upcoming)
            .chain(&self.order.history)
            .copied()
            .chain(self.index)
            .collect()
    }
    fn compact_queue(&mut self, retained: &std::collections::HashSet<usize>) {
        let mut remap = vec![0; self.queue.len()];
        let mut next = 0;
        let mut old = 0;
        self.queue.retain(|_| {
            let keep = retained.contains(&old);
            if keep {
                remap[old] = next;
                next += 1;
            }
            old += 1;
            keep
        });
        for index in self
            .order
            .context
            .iter_mut()
            .chain(&mut self.order.upcoming)
            .chain(&mut self.order.history)
        {
            *index = remap[*index];
        }
        self.index = self.index.map(|index| remap[index]);
        self.pending_index = self.pending_index.map(|index| remap[index]);
    }
    fn remember_current(&mut self) {
        if let Some(index) = self.index {
            self.order.history.push(index);
            // ponytail: Previous keeps 64 played occurrences; older manual rows can be reclaimed.
            if self.order.history.len() > 64 {
                self.order.history.remove(0);
            }
        }
    }
    pub fn move_manual(&mut self, from: usize, to: usize) {
        if from < self.order.manual_count {
            let index = self.order.upcoming.remove(from);
            let to = (if to > from { to - 1 } else { to }).min(self.order.manual_count - 1);
            self.order.upcoming.insert(to, index);
        }
    }
    pub fn set_shuffle(&mut self, enabled: bool) {
        self.local.shuffle = enabled;
        let rest = &mut self.order.upcoming[self.order.manual_count..];
        if enabled {
            rest.shuffle(&mut rand::rng());
            self.order.context.shuffle(&mut rand::rng());
        } else {
            rest.sort_unstable();
            self.order.context.sort_unstable();
        }
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
    pub fn jump(&mut self, position: usize) -> bool {
        let Some(index) = self.order.upcoming.get(position).copied() else {
            return false;
        };
        if !self.queue[index].available() {
            self.error = Some("Apple supplied no playback parameters for the next song. Choose another queue row to continue.".into());
            return false;
        }
        self.remember_current();
        self.order.upcoming.drain(..=position);
        self.order.manual_count = self.order.manual_count.saturating_sub(position + 1);
        self.select_pending(index);
        true
    }
    pub fn skip(&mut self, direction: i32) -> bool {
        if direction > 0 {
            if !self.order.upcoming.is_empty() {
                return self.jump(0);
            }
            if self.local.repeat == crate::player::RepeatMode::Context
                && !self.order.context.is_empty()
            {
                let index = self.order.context[0];
                if !self.queue[index].available() {
                    self.error = Some("Apple supplied no playback parameters for the next song. Choose another queue row to continue.".into());
                    return false;
                }
                self.order.upcoming = self.order.context[1..].to_vec();
                self.remember_current();
                self.select_pending(index);
                return true;
            }
        } else if let Some(index) = self.order.history.pop() {
            if let Some(current) = self.index {
                let manual = !self.order.context.contains(&current);
                self.order
                    .upcoming
                    .insert(if manual { 0 } else { self.order.manual_count }, current);
                self.order.manual_count += usize::from(manual);
            }
            if let Some(at) = self.order.upcoming.iter().position(|at| *at == index) {
                self.order.upcoming.remove(at);
                self.order.manual_count -= usize::from(at < self.order.manual_count);
            }
            self.select_pending(index);
            return true;
        }
        false
    }
    fn select_pending(&mut self, index: usize) {
        let paused = self.local.playback == Playback::Paused;
        self.pending_index = Some(index);
        self.pending_playback = paused.then_some(Playback::Paused);
        if self.index == Some(index) {
            self.local.track_sequence += 1;
        }
        self.select(index);
        self.local.position_ms = 0;
        self.local.position_at = None;
        self.local.playback = if paused {
            Playback::Paused
        } else {
            Playback::Loading
        };
    }
    pub fn intent(&mut self, command: Value) -> Value {
        self.request_generation += 1;
        json!({"type":"intent","generation":self.request_generation,"command":command})
    }
    pub fn toggle_play(&mut self) -> Value {
        if self.index.is_none() && self.jump(0) {
            return self.selection_command(true);
        }
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
                if event["status"] == 10 && self.pending_index.is_some() {
                    return None;
                }
                if let Some(index) = index {
                    if let Some(order) = event.get("order") {
                        let Ok(order) = serde_json::from_value::<QueueOrder>(order.clone()) else {
                            return None;
                        };
                        if order.validate(self.queue.len(), Some(index)).is_err() {
                            return None;
                        }
                        self.order = order;
                    }
                    self.select(index);
                }
                let playback = match event["status"].as_u64() {
                    Some(2) => Playback::Playing,
                    Some(3) => Playback::Paused,
                    Some(0) if index.is_some() => Playback::Paused,
                    Some(1 | 4 | 6 | 8) => Playback::Loading,
                    Some(10) => Playback::Paused,
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
                if event["status"] == 10 && self.error.is_none() {
                    let advance = if self.local.repeat == crate::player::RepeatMode::Track {
                        if let Some(index) = self.index {
                            self.select_pending(index);
                            true
                        } else {
                            false
                        }
                    } else {
                        self.skip(1)
                    };
                    if advance {
                        let command = self.selection_command(true);
                        return Some(self.intent(command));
                    }
                }
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
    fn queue_state() -> State {
        let songs = ["i.a", "i.b", "i.c"]
            .iter()
            .map(|id| {
                serde_json::from_value(json!({
            "kind":"library","id":id,"playParams":{"id":id,"kind":"song","isLibrary":true},
            "catalogId":null,"title":id,"artist":"Example","album":"Example","durationMs":180000,
        })).unwrap()
            })
            .collect();
        let mut state = State {
            songs,
            ..Default::default()
        };
        state.play(0).unwrap();
        state
    }
    #[test]
    fn manual_occurrences_precede_context_and_survive_new_contexts() {
        let mut state = queue_state();
        let uris = [
            state.songs[1].uri(),
            state.songs[2].uri(),
            state.songs[1].uri(),
        ];
        let debounce = std::time::Duration::from_millis(1500);
        assert_eq!(state.add_uris(&uris, 0, debounce, false), Some(3));
        assert_eq!(state.order.upcoming, [3, 4, 5, 1, 2]);
        assert!(state.add_uris(&uris, 3, debounce, false).is_none());
        state.move_manual(0, 3);
        assert_eq!(state.order.upcoming, [4, 5, 3, 1, 2]);
        assert!(state.jump(1));
        assert_eq!(state.index, Some(5));
        assert_eq!(state.order.upcoming, [3, 1, 2]);
        assert_eq!(state.order.manual_count, 1);
        let start = state
            .play_uris(&[state.songs[2].uri(), state.songs[0].uri()], 0)
            .unwrap();
        assert_eq!(start["items"][2]["id"], "i.b");
        assert_eq!(state.order.upcoming, [2, 1]);
        state.clear_manual();
        assert_eq!(state.order.upcoming, [1]);
        assert_eq!(state.order.manual_count, 0);
        assert_eq!(state.queue.len(), 2);
        let command = serde_json::from_value::<applifast_playback_probe::protocol::Command>(
            state.queue_command(),
        )
        .unwrap();
        assert!(command.validate().is_ok());
    }
    #[test]
    fn explicit_shuffle_next_stale_events_and_failed_rows_keep_the_queue() {
        let mut state = queue_state();
        let uri = state.songs[0].uri();
        state
            .add_uris(&[uri], 0, std::time::Duration::ZERO, false)
            .unwrap();
        state.set_shuffle(true);
        assert_eq!(state.order.upcoming[0], 3);
        let mut rest = state.order.upcoming[1..].to_vec();
        rest.sort_unstable();
        assert_eq!(rest, [1, 2]);
        let next = state.order.upcoming[0];
        assert!(state.skip(1));
        state.intent(json!({"type":"next"}));
        assert_eq!(state.index, Some(next));
        assert_eq!(state.order.manual_count, 0);
        let remaining = state.order.upcoming.clone();
        state.event(1, &json!({"type":"state","session":1,"requestGeneration":1,"index":0,"position":90,"status":2}));
        assert_eq!(state.index, Some(next));
        assert_eq!(state.order.upcoming, remaining);
        state.set_shuffle(false);
        let blocked = state.order.upcoming[0];
        state.queue[blocked].item.play_params = None;
        assert!(!state.skip(1));
        assert_eq!(state.order.upcoming[0], blocked);
        assert_eq!(state.index, Some(next));
        state.event(
            1,
            &json!({"type":"error","session":1,"requestGeneration":2,"message":"Unavailable"}),
        );
        assert_eq!(state.order.upcoming[0], blocked);
    }
    #[test]
    fn consumed_manual_rows_are_reclaimed_without_changing_occurrence_history() {
        let mut state = queue_state();
        let uri = state.songs[0].uri();
        for _ in 0..1100 {
            assert_eq!(
                state.add_uris(
                    std::slice::from_ref(&uri),
                    0,
                    std::time::Duration::ZERO,
                    true
                ),
                Some(1)
            );
            assert!(state.skip(1));
            assert_eq!(state.local.track.as_ref().unwrap().uri, uri);
            assert!(state.queue.len() <= 69);
            assert!(state.order.validate(state.queue.len(), state.index).is_ok());
        }
        assert!(state.skip(-1));
        assert_eq!(state.order.manual_count, 1);
        assert!(state.skip(1));
        assert!(state.order.validate(state.queue.len(), state.index).is_ok());
    }
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
