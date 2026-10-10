//! First Apple listening slice. UI state is separate from Spotify profile grants.
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::player::{LocalState, LocalTrack, Playback};
use applifast_playback_probe::protocol::{PlaybackItem, QueueOrder};
use rand::seq::SliceRandom;
pub mod cache;
pub mod models;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Read {
    Song(String),
    Home(HomeShelf),
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
    Search {
        serial: u64,
        library: bool,
    },
    SearchPage {
        serial: u64,
        library: bool,
        filter: crate::model::SearchFilter,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HomeShelf {
    Recent,
    Added,
    HeavyRotation,
    Recommendations,
}

impl HomeShelf {
    pub const ALL: [Self; 4] = [
        Self::Recent,
        Self::Added,
        Self::HeavyRotation,
        Self::Recommendations,
    ];

    pub fn path(self) -> &'static str {
        match self {
            Self::Recent => {
                "/v1/me/recent/played?types=albums,library-albums,playlists,library-playlists,artists&limit=10"
            }
            Self::Added => "/v1/me/library/recently-added",
            Self::HeavyRotation => "/v1/me/history/heavy-rotation?limit=10",
            Self::Recommendations => "/v1/me/recommendations?limit=10",
        }
    }
}

#[derive(Clone)]
pub struct HomeCard {
    pub name: String,
    pub subtitle: String,
    pub image: Option<String>,
    pub uri: String,
    pub page: Option<crate::model::Page>,
    pub playable: bool,
}

#[derive(Clone, Serialize, Deserialize)]
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
    /// Missing state is unknown, never inferred from library membership or ratings.
    #[serde(default)]
    pub in_favorites: Option<bool>,
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
    pub token_request: Option<u64>,
    pub filter: String,
    pub favorites_only: bool,
    pub home: std::collections::HashMap<HomeShelf, crate::model::Loadable<Vec<HomeCard>>>,
    pub queue: Vec<Song>,
    pub order: QueueOrder,
    recent_adds: std::collections::HashMap<String, Instant>,
    pub known_songs: std::collections::HashMap<String, Song>,
    pub storefront: String,
    pub reads: std::collections::HashMap<u64, (Read, u32)>,
    pub playlist_creates: std::collections::HashMap<u64, String>,
    pub playlist_confirms: std::collections::HashMap<
        String,
        crate::model::PagedList<crate::api::models::PlaylistItem>,
    >,
    pub playlist_recheck_at: Option<Instant>,
    pub playlist_cards: std::collections::HashSet<String>,
    pub writable_playlists: std::collections::HashSet<String>,
    pub pending_playlist_add: Option<crate::model::Action>,
    pub pending_album_playlist: Option<crate::model::Action>,
    pub playlist_appends: std::collections::HashMap<u64, (String, usize)>,
    pub next_reads: std::collections::HashMap<Read, (String, u32)>,
    pub pending_play: Option<crate::model::Action>,
    read_serial: u64,
    library_read_id: Option<u64>,
    pub index: Option<usize>,
    pub local: LocalState,
    pending_index: Option<usize>,
    pending_playback: Option<Playback>,
    request_generation: u64,
    pub account_tag: Option<String>,
    pub cache_checked: bool,
    cache_dirty: bool,
    cache_position_ms: u32,
    refresh_songs: Option<Vec<Song>>,
    restore_position: Option<u32>,
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
            token_request: None,
            filter: String::new(),
            favorites_only: false,
            home: Default::default(),
            queue: Vec::new(),
            order: QueueOrder::default(),
            recent_adds: Default::default(),
            known_songs: Default::default(),
            storefront: String::new(),
            reads: Default::default(),
            playlist_creates: Default::default(),
            playlist_confirms: Default::default(),
            playlist_recheck_at: None,
            playlist_cards: Default::default(),
            writable_playlists: Default::default(),
            pending_playlist_add: None,
            pending_album_playlist: None,
            playlist_appends: Default::default(),
            next_reads: Default::default(),
            pending_play: None,
            read_serial: 0,
            library_read_id: None,
            index: None,
            local: LocalState::default(),
            pending_index: None,
            pending_playback: None,
            request_generation: 0,
            account_tag: None,
            cache_checked: false,
            cache_dirty: false,
            cache_position_ms: 0,
            refresh_songs: None,
            restore_position: None,
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
        self.token_request = None;
        self.authorized = false;
        self.loading = false;
        self.songs.clear();
        self.favorites_only = false;
        self.home.clear();
        self.known_songs.clear();
        self.reads.clear();
        self.library_read_id = None;
        self.playlist_creates.clear();
        self.playlist_confirms.clear();
        self.playlist_recheck_at = None;
        self.playlist_cards.clear();
        self.writable_playlists.clear();
        self.pending_playlist_add = None;
        self.pending_album_playlist = None;
        self.playlist_appends.clear();
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
        self.account_tag = None;
        self.cache_checked = false;
        self.cache_dirty = false;
        self.refresh_songs = None;
        self.restore_position = None;
    }
    pub fn begin_token_import(&mut self) -> Option<u64> {
        if self.token_request.is_some() || (self.ready && self.loading && !self.authorized) {
            return None;
        }
        self.read_serial += 1;
        self.token_request = Some(self.read_serial);
        self.token_request
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
        let queue = self
            .songs_for_uris(uris)
            .map(|songs| songs.into_iter().cloned().collect::<Vec<_>>());
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
        self.cache_dirty = true;
        self.restore_position = None;
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
        let uris_to_add = uris
            .iter()
            .filter(|uri| album || !self.recent_adds.contains_key(*uri))
            .cloned()
            .collect::<Vec<_>>();
        let additions = self
            .songs_for_uris(&uris_to_add)
            .map(|songs| songs.into_iter().cloned().collect::<Vec<_>>());
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
            .or_else(|| self.queue.iter().find(|song| song.uri() == uri))
    }
    fn songs_for_uris(&self, uris: &[String]) -> Option<Vec<&Song>> {
        if uris.len() <= 1 {
            return uris.iter().map(|uri| self.find_song(uri)).collect();
        }
        // Index once per batch instead of allocating every library URI for every row.
        // Preserve find_song's precedence and first matching occurrence in each list.
        let mut lookup = self
            .known_songs
            .iter()
            .map(|(uri, song)| (uri.clone(), song))
            .collect::<std::collections::HashMap<_, _>>();
        for song in self.songs.iter().chain(&self.queue) {
            lookup.entry(song.uri()).or_insert(song);
        }
        uris.iter().map(|uri| lookup.get(uri).copied()).collect()
    }
    pub fn read(&mut self, target: Read, path: String, offset: u32) -> Value {
        self.read_serial += 1;
        self.reads.insert(self.read_serial, (target, offset));
        json!({"type":"request","id":self.read_serial,"path":path})
    }
    pub fn library_request(&mut self, next: Option<String>) -> Value {
        self.read_serial += 1;
        self.library_read_id = Some(self.read_serial);
        self.loading = true;
        self.error = None;
        json!({"type":"library", "id":self.read_serial, "next":next})
    }
    pub fn create_playlist(
        &mut self,
        name: &str,
        public: bool,
        uris: &[String],
    ) -> Result<(String, Value), String> {
        let items = self.playlist_items(uris)?;
        let id = self.read_serial + 1;
        let command = applifast_playback_probe::protocol::Command::CreatePlaylist {
            id,
            name: name.trim().into(),
            public,
            items,
        };
        command.validate()?;
        let temporary = format!("library.pending.{}.{}", self.session, id);
        self.read_serial = id;
        self.playlist_creates.insert(id, temporary.clone());
        Ok((
            temporary,
            serde_json::to_value(command).expect("playlist command serializes"),
        ))
    }
    pub fn playlist_items(&self, uris: &[String]) -> Result<Vec<PlaybackItem>, String> {
        self.songs_for_uris(uris)
            .map(|songs| songs.into_iter().map(|song| song.item.clone()).collect())
            .ok_or_else(|| "Reload the selected songs before saving this playlist.".to_owned())
    }
    pub fn append_playlist(
        &mut self,
        playlist: &str,
        uris: &[String],
        before: usize,
    ) -> Result<Value, String> {
        let id = self.read_serial + 1;
        let command = applifast_playback_probe::protocol::Command::AppendPlaylist {
            id,
            playlist: playlist.strip_prefix("library.").unwrap_or("").into(),
            items: self.playlist_items(uris)?,
        };
        command.validate()?;
        self.read_serial = id;
        self.playlist_appends.insert(id, (playlist.into(), before));
        Ok(serde_json::to_value(command).expect("playlist command serializes"))
    }
    pub fn remember_playlist_permissions(&mut self, rows: &[Value]) {
        for row in rows {
            if row["type"] != "library-playlists" {
                continue;
            }
            let Some(id) = row["id"]
                .as_str()
                .filter(|id| applifast_playback_probe::protocol::valid_library_playlist_id(id))
            else {
                continue;
            };
            let id = format!("library.{id}");
            if row["attributes"]["canEdit"] == true {
                self.writable_playlists.insert(id);
            } else {
                self.writable_playlists.remove(&id);
            }
        }
    }
    fn select(&mut self, index: usize) {
        if let Some(song) = self.queue.get(index) {
            if self.index != Some(index) {
                self.cache_dirty = true;
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
        if command["type"] != "volume" {
            self.cache_dirty = true;
        }
        self.restore_position = None;
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
                self.account_tag = event["accountTag"]
                    .as_str()
                    .filter(|tag| cache::valid_tag(tag))
                    .map(str::to_owned);
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
                    return Some(self.library_request(None));
                }
            }
            Some("authorized") => {
                self.account_tag = event["accountTag"]
                    .as_str()
                    .filter(|tag| cache::valid_tag(tag))
                    .map(str::to_owned);
                if let Some(storefront) = event["storefront"].as_str().filter(|value| {
                    value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_lowercase())
                }) {
                    self.storefront = storefront.to_owned();
                }
                self.authorized = true;
                self.local.connected = true;
                self.loading = true;
                return Some(self.library_request(None));
            }
            Some("signedOut") => self.clear_account(),
            Some("library") => {
                if self.library_read_id.is_none() || event["id"].as_u64() != self.library_read_id {
                    return None;
                }
                self.library_read_id = None;
                self.loading = false;
                if event["error"].is_string() {
                    self.error = Some("Apple Music could not load songs. Check sign-in and connection, then refresh Songs.".into());
                    self.refresh_songs = None;
                    return None;
                }
                let parsed: Result<Vec<Song>, _> = serde_json::from_value(event["items"].clone());
                match parsed {
                    Ok(songs) if songs.iter().all(|song| song.item.validate().is_ok()) => {
                        let next = event["next"].as_str().map(str::to_owned);
                        if let Some(fresh) = &mut self.refresh_songs {
                            if songs.is_empty() && next.is_some() {
                                self.error = Some("Apple returned an empty continuation page. Reload Songs to retry.".into());
                                self.refresh_songs = None;
                                return None;
                            }
                            fresh.extend(songs);
                            // Keep cached rows visible until their loaded span is refreshed.
                            if next.is_some() && fresh.len() < self.songs.len() {
                                self.loading = true;
                                return Some(self.library_request(next));
                            }
                            self.songs = self.refresh_songs.take().unwrap_or_default();
                        } else {
                            self.songs.extend(songs);
                        }
                        self.next = next;
                        self.cache_dirty = true;
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
                let position_ms =
                    (event["position"].as_f64().unwrap_or(0.0).max(0.0) * 1000.0) as u32;
                if let Some(expected) = self.restore_position {
                    if playback != Playback::Paused || position_ms.abs_diff(expected) > 1000 {
                        return None;
                    }
                    self.restore_position = None;
                }
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
                if event.get("requestGeneration").is_none() {
                    self.loading = false;
                    self.refresh_songs = None;
                    self.library_read_id = None;
                }
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
                self.restore_position = None;
                self.local.playback = Playback::Paused;
                self.local.position_at = None;
            }
            _ => {}
        }
        None
    }
    pub fn restore_cache(&mut self, snapshot: Option<cache::Snapshot>) -> Option<Value> {
        if !self.authorized || self.cache_checked {
            return None;
        }
        self.cache_checked = true;
        if self.request_generation != 0 {
            return None;
        }
        let snapshot = snapshot.filter(|snapshot| {
            self.account_tag
                .as_deref()
                .is_some_and(|tag| snapshot.valid_for(tag, &self.storefront))
        })?;
        self.songs = snapshot.songs;
        self.next = snapshot.next;
        self.refresh_songs = Some(Vec::new());
        self.queue = snapshot.queue;
        self.order = snapshot.order;
        self.index = snapshot.index;
        if let Some(index) = self.index {
            self.select(index);
        }
        self.local.position_ms = self
            .local
            .track
            .as_ref()
            .filter(|song| song.duration_ms > 0)
            .map_or(snapshot.position_ms, |song| {
                snapshot.position_ms.min(song.duration_ms)
            });
        self.local.position_at = None;
        self.local.playback = if self.index.is_some() {
            Playback::Paused
        } else {
            Playback::Stopped
        };
        self.local.shuffle = snapshot.shuffle;
        self.local.repeat = match snapshot.repeat {
            1 => crate::player::RepeatMode::Track,
            2 => crate::player::RepeatMode::Context,
            _ => crate::player::RepeatMode::Off,
        };
        let request = json!({"type":"restore","items":self.queue.iter().map(|song| &song.item).collect::<Vec<_>>(),"index":self.index,"order":self.order,"seconds":f64::from(self.local.position_ms)/1000.0,"shuffle":snapshot.shuffle,"repeat":snapshot.repeat});
        let request = self.intent(request);
        self.restore_position = self.index.map(|_| self.local.position_ms);
        self.pending_index = self.index;
        self.pending_playback = self.index.map(|_| Playback::Paused);
        Some(request)
    }
    pub fn cache_snapshot(&mut self, force: bool) -> Option<cache::Snapshot> {
        let tag = self.account_tag.clone()?;
        if !self.authorized || !self.cache_checked {
            return None;
        }
        let elapsed = self.local.position_at.map_or(0, |at| {
            at.elapsed().as_millis().min(u128::from(u32::MAX)) as u32
        });
        let position = self.local.position_ms.saturating_add(elapsed);
        if !force && !self.cache_dirty && position.abs_diff(self.cache_position_ms) < 15_000 {
            return None;
        }
        self.cache_dirty = false;
        self.cache_position_ms = position;
        Some(cache::Snapshot {
            version: 1,
            account_tag: tag,
            storefront: self.storefront.clone(),
            songs: self.songs.clone(),
            next: self.next.clone(),
            queue: self.queue.clone(),
            order: self.order.clone(),
            index: self.index,
            position_ms: position,
            shuffle: self.local.shuffle,
            repeat: match self.local.repeat {
                crate::player::RepeatMode::Off => 0,
                crate::player::RepeatMode::Track => 1,
                crate::player::RepeatMode::Context => 2,
            },
        })
    }
    pub fn refresh_library(&mut self) -> Value {
        self.refresh_songs = Some(Vec::new());
        self.error = None;
        self.library_request(None)
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
                in_favorites: Some(index % 3 == 0),
            });
        }
        for (index, shelf) in HomeShelf::ALL.into_iter().enumerate() {
            state.home.insert(
                shelf,
                crate::model::Loadable::Loaded(
                    state
                        .songs
                        .iter()
                        .skip(index)
                        .take(4)
                        .map(|song| HomeCard {
                            name: song.title.clone(),
                            subtitle: song.artist.clone(),
                            image: song.artwork.clone(),
                            uri: song.uri(),
                            page: None,
                            playable: song.available(),
                        })
                        .collect(),
                ),
            );
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
    fn saved_library_and_occurrences_restore_paused_and_reject_other_grants() {
        let mut source = queue_state();
        source.authorized = true;
        source.cache_checked = true;
        source.account_tag = Some("a".repeat(64));
        source.storefront = "us".into();
        source.songs[0].in_favorites = Some(true);
        source.songs[1].in_favorites = Some(false);
        let catalog: Song = serde_json::from_value(json!({"kind":"catalog","id":"123","playParams":{"id":"123","kind":"song"},"title":"Catalog","artist":"Artist","album":"Album","durationMs":180000,"catalogId":"123"})).unwrap();
        let catalog_uri = catalog.uri();
        source.known_songs.insert(catalog_uri.clone(), catalog);
        source.add_uris(
            &[
                source.songs[0].uri(),
                source.songs[0].uri(),
                catalog_uri.clone(),
            ],
            0,
            std::time::Duration::ZERO,
            true,
        );
        source.local.playback = Playback::Paused;
        source.seek(42_000);
        source.songs[2].item.play_params = None;
        let snapshot = source.cache_snapshot(true).unwrap();
        let root = std::env::temp_dir().join(format!(
            "applifast-cache-{}-{:016x}",
            std::process::id(),
            rand::random::<u64>()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("apple-session.json");
        #[cfg(windows)]
        let previous_reader = {
            let mut previous = snapshot.clone();
            previous.position_ms = 0;
            previous.save(&path).unwrap();
            std::fs::File::open(&path).unwrap()
        };
        snapshot.save(&path).unwrap();
        #[cfg(windows)]
        assert_eq!(
            serde_json::from_reader::<_, cache::Snapshot>(previous_reader)
                .unwrap()
                .position_ms,
            0
        );
        assert!(cache::Snapshot::load(&path, &"b".repeat(64), "us").is_none());
        assert!(cache::Snapshot::load(&path, &"a".repeat(64), "gb").is_none());
        let snapshot = cache::Snapshot::load(&path, &"a".repeat(64), "us").unwrap();
        let mut restored = State {
            authorized: true,
            account_tag: source.account_tag.clone(),
            storefront: "us".into(),
            ..Default::default()
        };
        let command = restored.restore_cache(Some(snapshot.clone())).unwrap();
        let parsed =
            serde_json::from_value::<applifast_playback_probe::protocol::Command>(command.clone())
                .unwrap();
        assert!(parsed.validate().is_ok());
        assert_eq!(restored.local.playback, Playback::Paused);
        assert_eq!(restored.local.position_ms, 42_000);
        assert_eq!(restored.songs[0].in_favorites, Some(true));
        assert_eq!(restored.songs[1].in_favorites, Some(false));
        assert_eq!(restored.songs[2].in_favorites, None);
        assert_eq!(restored.order.upcoming, source.order.upcoming);
        assert_eq!(restored.order.manual_count, 3);
        assert_eq!(restored.queue[3].uri(), restored.queue[4].uri());
        assert_eq!(command["command"]["items"][0]["playParams"]["id"], "i.a");
        assert!(!restored.songs[2].available());
        assert!(restored.known_songs.is_empty());
        assert_eq!(restored.find_song(&catalog_uri).unwrap().item.id, "123");
        restored.event(1, &json!({"type":"state","session":1,"requestGeneration":1,"index":0,"position":0,"status":3}));
        assert_eq!(restored.local.position_ms, 42_000);
        restored.event(1, &json!({"type":"state","session":1,"requestGeneration":1,"index":0,"position":42,"status":3}));
        assert!(restored.restore_position.is_none());
        assert!(restored.restore_cache(Some(snapshot.clone())).is_none());
        restored.cache_snapshot(true).unwrap();
        restored.play_uris(&[restored.songs[1].uri()], 0).unwrap();
        assert!(restored.restore_position.is_none());
        let replaced = restored.cache_snapshot(false).unwrap();
        assert_eq!(replaced.queue[0].item.id, "i.b");
        restored.clear_account();
        assert!(!restored.favorites_only);
        assert!(restored.restore_cache(Some(snapshot.clone())).is_none());
        assert!(restored.cache_snapshot(true).is_none());
        let mut invalid = snapshot;
        invalid.order.upcoming.push(9999);
        assert!(invalid.save(&path).is_err());
        std::fs::write(&path, b"not JSON").unwrap();
        assert!(cache::Snapshot::load(&path, &"a".repeat(64), "us").is_none());
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn cached_rows_stay_until_refresh_completes_and_cannot_undo_a_new_queue() {
        let mut state = queue_state();
        let request = state.refresh_library();
        let row = serde_json::to_value(&state.songs[0]).unwrap();
        let next = state.event(1, &json!({"type":"library","id":request["id"],"session":1,"items":[row.clone()],"next":"/v1/me/library/songs?offset=100"})).unwrap();
        assert_ne!(request["id"], next["id"]);
        assert_eq!(state.songs.len(), 3);
        state.event(
            1,
            &json!({"type":"library","id":next["id"],"session":1,"items":[row],"next":null}),
        );
        assert_eq!(state.songs.len(), 2);
        assert_eq!(state.queue.len(), 3);
        state.authorized = true;
        assert!(state.restore_cache(None).is_none());
        assert!(state.cache_checked);
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
    fn batch_song_lookup_preserves_precedence_identity_and_atomic_failures() {
        let mut state = queue_state();
        let mut known = state.songs[0].clone();
        known.title = "Known upload".into();
        known.item.play_params =
            Some(json!({"id":"i.a","kind":"song","isLibrary":true,"assetId":"uploaded"}));
        state.known_songs.insert(known.uri(), known.clone());
        let mut duplicate = state.songs[1].clone();
        duplicate.title = "Later library copy".into();
        state.songs.push(duplicate);
        state.queue[1].title = "Queue copy".into();
        let mut catalog = known.clone();
        catalog.item.kind = applifast_playback_probe::protocol::ItemKind::Catalog;
        catalog.item.play_params = Some(json!({"id":"i.a","kind":"song"}));
        catalog.title = "Distinct catalog ID".into();
        state.queue.push(catalog.clone());
        let uris = [
            known.uri(),
            state.songs[1].uri(),
            known.uri(),
            catalog.uri(),
        ];
        for (uri, song) in uris.iter().zip(state.songs_for_uris(&uris).unwrap()) {
            assert_eq!(song.title, state.find_song(uri).unwrap().title);
        }
        let items = state.playlist_items(&uris).unwrap();
        assert_eq!(
            serde_json::to_value(&items[0]).unwrap(),
            serde_json::to_value(&known.item).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&items[2]).unwrap(),
            serde_json::to_value(&known.item).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&items[3]).unwrap(),
            serde_json::to_value(&catalog.item).unwrap()
        );
        state.play_uris(&uris, 0).unwrap();
        assert_eq!(state.queue[0].catalog_id, None);
        assert_eq!(state.queue[1].title, "i.b");
        assert_eq!(
            serde_json::to_value(&state.queue[2].item).unwrap(),
            serde_json::to_value(&known.item).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&state.queue[3].item).unwrap(),
            serde_json::to_value(&catalog.item).unwrap()
        );
        assert_eq!(
            state.add_uris(&uris, 0, std::time::Duration::ZERO, false),
            Some(4)
        );
        let before = state.queue_command();
        let missing = [known.uri(), "apple:track:library.missing".into()];
        assert!(state.play_uris(&missing, 0).is_none());
        assert!(
            state
                .add_uris(&missing, 0, std::time::Duration::ZERO, false)
                .is_none()
        );
        assert!(state.playlist_items(&missing).is_err());
        assert_eq!(state.queue_command(), before);
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
            assert!(ready["accountTag"].as_str().is_some_and(cache::valid_tag));
            let order = QueueOrder {
                upcoming: vec![1],
                manual_count: 1,
                context: vec![0],
                history: Vec::new(),
            };
            host.send(json!({"type":"intent","generation":4,"command":{"type":"restore","items":[song.item,song.item],"index":0,"order":order,"seconds":30,"shuffle":false,"repeat":0}}).to_string());
            await_event("state", &|event| {
                event["requestGeneration"] == 4
                    && event["status"] == 3
                    && event["actualPosition"].as_f64().unwrap_or(0.0) >= 29.0
                    && event["queueLength"] == 2
                    && event["order"]["manualCount"] == 1
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
        let request = state.library_request(None);
        state.event(
            1,
            &json!({"type":"library","id":request["id"],"session":1,"items":[row.clone(),row],"next":null}),
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
        let request = state.library_request(None);
        state.event(1, &json!({"type":"library","id":request["id"],"session":1,"items":[row.clone()],"next":"/v1/me/library/songs?offset=100"}));
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
        let request = state.library_request(None);
        state.event(
            1,
            &json!({"type":"library","id":request["id"],"session":2,"items":[row],"next":null}),
        );
        assert_eq!(state.songs.len(), 1);
        assert!(state.next.is_none());
    }

    #[test]
    fn library_replies_cannot_undo_refresh_or_interrupt_playback() {
        let mut state = queue_state();
        state.local.playback = Playback::Playing;
        let mut fresh = serde_json::to_value(&state.songs[0]).unwrap();
        fresh["inFavorites"] = json!(true);
        let old = state.library_request(None);
        let request = state.refresh_library();
        for reply in [
            json!({"type":"library","id":old["id"],"items":[],"next":null}),
            json!({"type":"library","id":old["id"],"error":"secret SDK response"}),
            json!({"type":"library","items":[],"next":null}),
        ] {
            assert!(state.event(1, &reply).is_none());
            assert!(state.loading);
            assert!(state.error.is_none());
            assert_eq!(state.songs.len(), 3);
        }
        let reply = json!({"type":"library","id":request["id"],"items":[fresh],"next":null});
        state.event(1, &reply);
        assert_eq!(state.songs.len(), 1);
        assert_eq!(state.songs[0].in_favorites, Some(true));
        assert!(!state.loading);
        state.event(
            1,
            &json!({"type":"library","id":old["id"],"items":[],"next":null}),
        );
        state.event(1, &reply); // A duplicate page cannot duplicate library rows.
        assert_eq!(state.songs.len(), 1);
        let failed = state.library_request(state.next.clone());
        state.event(
            1,
            &json!({"type":"library","id":failed["id"],"error":"secret SDK response"}),
        );
        assert!(!state.loading);
        assert!(!state.error.as_ref().unwrap().contains("secret"));
        assert_eq!(state.songs[0].in_favorites, Some(true));
        assert_eq!(state.queue.len(), 3);
        assert_eq!(state.local.playback, Playback::Playing);
        let retry = state.refresh_library();
        state.event(1, &json!({"type":"error","requestGeneration":state.request_generation,"message":"Playback failed"}));
        assert!(
            state.loading,
            "Playback failure cannot cancel a metadata refresh"
        );
        assert_eq!(state.local.playback, Playback::Paused);
        state.event(1, &json!({"type":"library","id":retry["id"],"items":[serde_json::to_value(&state.songs[0]).unwrap()],"next":null}));
        assert!(!state.loading);
        assert_eq!(
            state.songs.len(),
            1,
            "Refresh still replaces its cached span"
        );
        assert_eq!(state.queue.len(), 3);
        state.clear_account();
        state.event(1, &reply);
        assert!(state.songs.is_empty());
    }
}
