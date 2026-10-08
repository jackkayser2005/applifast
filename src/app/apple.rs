//! Bind Apple reads to the existing pages, tables and navigation.
use super::*;
use crate::api::models::{Page as ApiPage, SavedAlbum};
use crate::apple::{Read, models};
use serde_json::Value;

fn page<T>(items: Vec<T>, data: &Value, offset: u32) -> ApiPage<T> {
    let next = data["next"]
        .as_str()
        .filter(|path| applifast_playback_probe::protocol::valid_read_path(path))
        .map(str::to_owned);
    let consumed = data["data"].as_array().map_or(items.len(), Vec::len) as u32;
    ApiPage {
        items,
        offset,
        limit: consumed,
        total: data["meta"]["total"]
            .as_u64()
            .map(|value| value.min(u64::from(u32::MAX)) as u32)
            .unwrap_or(offset + consumed + u32::from(next.is_some())),
        next,
    }
}

impl App {
    pub(super) fn sync_apple_library(&mut self) {
        let Some(apple) = &self.apple else { return };
        self.library.liked.items = apple
            .songs
            .iter()
            .map(|song| crate::api::models::SavedTrack {
                added_at: None,
                track: song.track(),
            })
            .collect();
        self.library.liked.loaded_once = !apple.songs.is_empty() || !apple.loading;
        self.library.liked.loading = apple.loading;
        self.library.liked.total = Some(apple.songs.len() as u32);
        self.library.liked.next_offset = apple.next.as_ref().map(|_| apple.songs.len() as u32);
        self.library.liked.revision += 1;
        for saved in &self.library.liked.items {
            if let Some(id) = &saved.track.id {
                self.track_cache.insert(id.clone(), saved.track.clone());
            }
        }
    }
    pub(super) fn apple_queue_add(&mut self, uris: &[String], position: usize, album: bool) {
        let Some(apple) = &mut self.apple else { return };
        if !apple.authorized {
            return;
        }
        if let Some(count) = apple.add_uris(uris, position, QUEUE_ADD_DEBOUNCE, album) {
            let request = apple.intent(apple.queue_command());
            self.backend.send(Command::AppleSend(request.to_string()));
            self.sync_apple_queue();
            self.queued_toast(count);
        }
    }
    pub(super) fn apple_queue_album(&mut self, uri: &str, label: &str) {
        if self
            .last_album_queue
            .as_ref()
            .is_some_and(|(previous, at)| previous == uri && at.elapsed() < QUEUE_ADD_DEBOUNCE)
        {
            return;
        }
        let Some(id) = util::uri_id(uri).map(str::to_owned) else {
            return;
        };
        self.last_album_queue = Some((uri.into(), Instant::now()));
        self.album_queue_serial = self.album_queue_serial.wrapping_add(1);
        self.pending_album_queues.insert(
            self.album_queue_serial,
            PendingAlbumQueue {
                id: id.clone(),
                label: label.into(),
                target: Target::Local,
                offset: 0,
                tracks: Vec::new(),
            },
        );
        self.apple_ensure_loaded(Page::Album(id));
        self.apple_finish_album_queues();
    }
    pub(super) fn apple_finish_album_queues(&mut self) {
        let pending = self
            .pending_album_queues
            .iter()
            .map(|(request, album)| (*request, album.id.clone()))
            .collect::<Vec<_>>();
        for (request, id) in pending {
            let Some(page) = self.album_pages.get(&id) else {
                continue;
            };
            if page.tracks.loading {
                continue;
            }
            if let Some(error) = &page.tracks.error {
                let error = error.clone();
                self.pending_album_queues.remove(&request);
                self.toast_error(error);
            } else if page.tracks.is_complete() {
                let uris = page
                    .tracks
                    .items
                    .iter()
                    .map(|track| track.uri.clone())
                    .collect::<Vec<_>>();
                self.pending_album_queues.remove(&request);
                let position = self
                    .apple
                    .as_ref()
                    .map_or(0, |apple| apple.order.manual_count);
                self.apple_queue_add(&uris, position, true);
            } else {
                self.apple_load_more(Page::Album(id));
            }
        }
    }
    pub(super) fn apple_evict_songs(&mut self) {
        let Some(apple) = &mut self.apple else {
            return;
        };
        let mut needed: HashSet<_> = self
            .album_pages
            .values()
            .flat_map(|page| page.tracks.items.iter().map(|track| track.uri.as_str()))
            .chain(self.playlist_pages.values().flat_map(|page| {
                page.items
                    .items
                    .iter()
                    .filter_map(|item| item.playable().map(PlayableItem::uri))
            }))
            .chain(
                self.artist_pages
                    .values()
                    .filter_map(|page| page.top_tracks.get())
                    .flat_map(|tracks| tracks.iter().map(|track| track.uri.as_str())),
            )
            .chain(
                self.search
                    .results
                    .get()
                    .and_then(|results| results.tracks.as_ref())
                    .into_iter()
                    .flat_map(|page| page.items.iter().map(|track| track.uri.as_str())),
            )
            .map(str::to_owned)
            .collect();
        needed.extend(apple.queue.iter().map(crate::apple::Song::uri));
        apple.known_songs.retain(|uri, _| {
            needed.contains(uri)
                || util::uri_id(uri).is_some_and(|id| self.track_cache.contains_key(id))
        });
    }
    pub(super) fn apple_finish_pending_play(&mut self) {
        let Some(apple) = &mut self.apple else {
            return;
        };
        let Some(action) = &apple.pending_play else {
            return;
        };
        let page = match action {
            Action::PlayContext { uri, .. } | Action::ShufflePlay(uri) => Page::from_uri(uri),
            _ => None,
        };
        if page.is_none_or(|page| {
            !apple
                .reads
                .values()
                .any(|(target, _)| read_for_page(target, &page))
        }) && let Some(action) = apple.pending_play.take()
        {
            self.actions.push(action);
        }
    }
    pub(super) fn apple_artist_filters(&mut self, id: &str) {
        let Some(artist) = self.artist_pages.get_mut(id) else {
            return;
        };
        let Some(all) = artist.albums.get("album,single,compilation") else {
            return;
        };
        let items = all.items.clone();
        let loaded = all.loaded_once;
        let next = all.next_offset;
        let loading = all.loading;
        for (key, kind) in [("album", "album"), ("single", "single")] {
            let list = artist.albums.entry(key.into()).or_default();
            list.items = items
                .iter()
                .filter(|album| album.album_type.as_deref() == Some(kind))
                .cloned()
                .collect();
            list.total = Some(list.items.len() as u32);
            list.loaded_once = loaded;
            list.loading = loading;
            list.next_offset = next;
            list.revision += 1;
        }
        let list = artist.albums.entry("appears_on".into()).or_default();
        list.error = Some("Apple does not expose this library artist category here.".into());
        list.loading = false;
    }
    pub(super) fn sync_apple_queue(&mut self) {
        let Some(apple) = &self.apple else {
            return;
        };
        self.queue = Loadable::Loaded(Queue {
            currently_playing: apple
                .index
                .and_then(|index| apple.queue.get(index))
                .map(|song| PlayableItem::Track(song.track())),
            queue: apple
                .order
                .upcoming
                .iter()
                .filter_map(|index| apple.queue.get(*index))
                .map(|song| PlayableItem::Track(song.track()))
                .collect(),
        });
        self.manual_queue = apple.order.upcoming[..apple.order.manual_count]
            .iter()
            .filter_map(|index| apple.queue.get(*index))
            .map(crate::apple::Song::uri)
            .collect();
        self.session_dirty = true;
    }
    fn apple_read(&mut self, target: Read, path: String, offset: u32) {
        let Some(apple) = &mut self.apple else { return };
        if !apple.authorized || apple.reads.values().any(|(pending, _)| pending == &target) {
            return;
        }
        if !applifast_playback_probe::protocol::valid_read_path(&path) {
            return;
        }
        let request = apple.read(target, path, offset);
        self.backend.send(Command::AppleSend(request.to_string()));
    }
    fn apple_resource_path(&self, resource: &str, id: &str) -> Option<String> {
        let (kind, id) = id.split_once('.')?;
        let apple = self.apple.as_ref()?;
        let prefix = match kind {
            "library" => "/v1/me/library".to_owned(),
            "catalog" if !apple.storefront.is_empty() => {
                format!("/v1/catalog/{}", apple.storefront)
            }
            _ => return None,
        };
        let path = format!("{prefix}/{resource}/{id}");
        applifast_playback_probe::protocol::valid_read_path(&path).then_some(path)
    }
    pub(super) fn apple_ensure_loaded(&mut self, page: Page) {
        if !self.account_ready() {
            return;
        }
        if self.library.playlists.needs_load() {
            self.library.playlists = Loadable::Loading;
            self.apple_read(
                Read::Playlists,
                "/v1/me/library/playlists?limit=100".into(),
                0,
            );
        }
        match page {
            Page::Albums if !self.library.albums.loaded_once && !self.library.albums.loading => {
                self.library.albums.loading = true;
                self.apple_read(Read::Albums, "/v1/me/library/albums?limit=100".into(), 0);
            }
            Page::Artists if !self.library.artists.loaded_once && !self.library.artists.loading => {
                self.library.artists.loading = true;
                self.apple_read(Read::Artists, "/v1/me/library/artists?limit=100".into(), 0);
            }
            Page::Album(id) if !self.album_pages.contains_key(&id) => {
                let Some(path) = self.apple_resource_path("albums", &id) else {
                    return;
                };
                let entry = self.album_pages.entry(id.clone()).or_default();
                entry.album = Loadable::Loading;
                entry.tracks.loading = true;
                self.apple_read(Read::Album(id.clone()), path.clone(), 0);
                self.apple_read(Read::AlbumTracks(id), format!("{path}/tracks?limit=100"), 0);
            }
            Page::Playlist(id) if !self.playlist_pages.contains_key(&id) => {
                let Some(path) = self.apple_resource_path("playlists", &id) else {
                    return;
                };
                let entry = self.playlist_pages.entry(id.clone()).or_default();
                entry.playlist = Loadable::Loading;
                entry.items.loading = true;
                self.apple_read(Read::Playlist(id.clone()), path.clone(), 0);
                self.apple_read(
                    Read::PlaylistTracks(id),
                    format!("{path}/tracks?limit=100"),
                    0,
                );
            }
            Page::Artist(id) if !self.artist_pages.contains_key(&id) => {
                let Some(path) = self.apple_resource_path("artists", &id) else {
                    return;
                };
                let entry = self.artist_pages.entry(id.clone()).or_default();
                entry.artist = Loadable::Loading;
                entry.related = Loadable::Loaded(Vec::new());
                entry.top_tracks = if id.starts_with("catalog.") {
                    Loadable::Loading
                } else {
                    Loadable::Loaded(Vec::new())
                };
                entry
                    .albums
                    .entry("album,single,compilation".into())
                    .or_default()
                    .loading = true;
                self.apple_read(Read::Artist(id.clone()), path.clone(), 0);
                self.apple_read(
                    Read::ArtistAlbums(id.clone()),
                    format!("{path}/albums?limit=100"),
                    0,
                );
                if id.starts_with("catalog.") {
                    self.apple_read(
                        Read::ArtistSongs(id),
                        format!("{path}/view/top-songs?limit=20"),
                        0,
                    );
                }
            }
            _ => {}
        }
    }
    pub(super) fn apple_load_more(&mut self, page: Page) {
        let initial = match &page {
            Page::Albums => !self.library.albums.loaded_once,
            Page::Artists => !self.library.artists.loaded_once,
            _ => false,
        };
        if initial {
            self.apple_ensure_loaded(page);
            return;
        }
        let target = match page {
            Page::Albums => Read::Albums,
            Page::Artists => Read::Artists,
            Page::Album(id) => Read::AlbumTracks(id),
            Page::Playlist(id) => Read::PlaylistTracks(id),
            Page::Artist(id) => Read::ArtistAlbums(id),
            _ => return,
        };
        if let Some((path, offset)) = self
            .apple
            .as_ref()
            .and_then(|apple| apple.next_reads.get(&target).cloned())
        {
            self.apple_read(target, path, offset);
        }
    }
    pub(super) fn apple_context_uris(&self, uri: &str) -> Vec<String> {
        if uri == "apple:collection:library" {
            return self.apple.as_ref().map_or_else(Vec::new, |apple| {
                apple.songs.iter().map(crate::apple::Song::uri).collect()
            });
        }
        match Page::from_uri(uri) {
            Some(Page::Album(id)) => self.album_pages.get(&id).map_or_else(Vec::new, |page| {
                page.tracks
                    .items
                    .iter()
                    .map(|track| track.uri.clone())
                    .collect()
            }),
            Some(Page::Playlist(id)) => {
                self.playlist_pages.get(&id).map_or_else(Vec::new, |page| {
                    page.items
                        .items
                        .iter()
                        .filter_map(|item| item.playable().map(|item| item.uri().to_owned()))
                        .collect()
                })
            }
            Some(Page::Artist(id)) => self
                .artist_pages
                .get(&id)
                .and_then(|page| page.top_tracks.get())
                .map_or_else(Vec::new, |tracks| {
                    tracks.iter().map(|track| track.uri.clone()).collect()
                }),
            _ => Vec::new(),
        }
    }
    pub(super) fn apple_search(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        let term = urlencoding::encode(query);
        let serial = self.search.serial;
        self.apple_read(Read::Search {serial,library:true}, format!("/v1/me/library/search?term={term}&types=library-songs,library-albums,library-artists,library-playlists&limit=25"),0);
        if let Some(storefront) = self
            .apple
            .as_ref()
            .map(|apple| apple.storefront.clone())
            .filter(|value| !value.is_empty())
        {
            self.apple_read(Read::Search {serial,library:false}, format!("/v1/catalog/{storefront}/search?term={term}&types=songs,albums,artists,playlists&limit=25"),0);
        }
    }
    pub(super) fn apple_response(&mut self, value: &Value) {
        let Some((target, offset)) = value["id"]
            .as_u64()
            .and_then(|id| self.apple.as_mut()?.reads.remove(&id))
        else {
            return;
        };
        if let Some(error) = value["error"].as_str() {
            match &target {
                Read::Playlists => self.library.playlists = Loadable::Failed(error.into()),
                Read::Albums => {
                    self.library.albums.loading = false;
                    self.library.albums.error = Some(error.into());
                }
                Read::Artists => {
                    self.library.artists.loading = false;
                    self.library.artists.error = Some(error.into());
                }
                Read::Album(id) => {
                    if let Some(page) = self.album_pages.get_mut(id) {
                        page.album = Loadable::Failed(error.into());
                    }
                }
                Read::Playlist(id) => {
                    if let Some(page) = self.playlist_pages.get_mut(id) {
                        page.playlist = Loadable::Failed(error.into());
                    }
                }
                Read::AlbumTracks(id) => {
                    if let Some(page) = self.album_pages.get_mut(id) {
                        page.tracks.loading = false;
                        page.tracks.error = Some(error.into());
                    }
                }
                Read::PlaylistTracks(id) => {
                    if let Some(page) = self.playlist_pages.get_mut(id) {
                        page.items.loading = false;
                        page.items.error = Some(error.into());
                    }
                }
                Read::Artist(id) => {
                    if let Some(page) = self.artist_pages.get_mut(id) {
                        page.artist = Loadable::Failed(error.into());
                    }
                }
                Read::ArtistSongs(id) => {
                    if let Some(page) = self.artist_pages.get_mut(id) {
                        page.top_tracks = Loadable::Failed(error.into());
                    }
                }
                Read::ArtistAlbums(id) => {
                    if let Some(page) = self.artist_pages.get_mut(id) {
                        let list = page
                            .albums
                            .entry("album,single,compilation".into())
                            .or_default();
                        list.loading = false;
                        list.error = Some(error.into());
                    }
                }
                Read::Search { serial, .. } if *serial == self.search.serial => {
                    self.search.error = Some(error.into());
                    self.search.catalogue_pending = false;
                    self.search.results.refresh::<String>(Err(error.into()));
                }
                _ => {}
            }
            return;
        }
        let data = &value["data"];
        if let Read::Search { serial, library } = target {
            if serial != self.search.serial {
                return;
            }
            let prefix = if library { "library-" } else { "" };
            let results = &data["results"];
            let tracks = self.apple_tracks(&results[format!("{prefix}songs")]);
            let fresh = crate::api::models::SearchResults {
                tracks: Some(page(tracks, &results[format!("{prefix}songs")], 0)),
                albums: Some(page(
                    resources(&results[format!("{prefix}albums")])
                        .iter()
                        .map(models::album)
                        .collect(),
                    &results[format!("{prefix}albums")],
                    0,
                )),
                artists: Some(page(
                    resources(&results[format!("{prefix}artists")])
                        .iter()
                        .map(models::artist)
                        .collect(),
                    &results[format!("{prefix}artists")],
                    0,
                )),
                playlists: Some(page(
                    resources(&results[format!("{prefix}playlists")])
                        .iter()
                        .map(models::playlist)
                        .collect(),
                    &results[format!("{prefix}playlists")],
                    0,
                )),
                ..Default::default()
            };
            if self.search.results_serial != serial || self.search.results.get().is_none() {
                self.search.results = Loadable::Loaded(Default::default());
                self.search.results_serial = serial;
            }
            if let Some(held) = self.search.results.get_mut() {
                merge(&mut held.tracks, fresh.tracks);
                merge(&mut held.albums, fresh.albums);
                merge(&mut held.artists, fresh.artists);
                merge(&mut held.playlists, fresh.playlists);
            }
            self.search.catalogue_pending=self.apple.as_ref().is_some_and(|apple| apple.reads.values().any(|(target,_)| matches!(target,Read::Search {serial:pending,..} if *pending==serial)));
            return;
        }
        let rows = resources(data);
        self.apple.as_mut().unwrap().next_reads.remove(&target);
        if let Some(next) = data["next"]
            .as_str()
            .filter(|path| applifast_playback_probe::protocol::valid_read_path(path))
        {
            self.apple
                .as_mut()
                .unwrap()
                .next_reads
                .insert(target.clone(), (next.into(), offset + rows.len() as u32));
        }
        match target {
            Read::Playlists => {
                let playlists = rows.iter().map(models::playlist).collect::<Vec<_>>();
                if offset == 0 {
                    self.library.playlists = Loadable::Loaded(playlists);
                } else if let Some(held) = self.library.playlists.get_mut() {
                    held.extend(playlists);
                }
                if let Some((path, offset)) = self
                    .apple
                    .as_mut()
                    .unwrap()
                    .next_reads
                    .remove(&Read::Playlists)
                {
                    self.apple_read(Read::Playlists, path, offset);
                }
            }
            Read::Albums => self.library.albums.absorb(
                offset,
                page(
                    rows.iter()
                        .map(|row| SavedAlbum {
                            added_at: None,
                            album: models::album(row),
                        })
                        .collect(),
                    data,
                    offset,
                ),
            ),
            Read::Artists => {
                let list = &mut self.library.artists;
                if offset == 0 {
                    list.items.clear();
                }
                list.items.extend(rows.iter().map(models::artist));
                list.loaded_once = true;
                list.loading = false;
                list.error = None;
                list.complete = data["next"].as_str().is_none();
            }
            Read::Album(id) => {
                if let Some(row) = rows.first() {
                    self.album_pages.entry(id).or_default().album =
                        Loadable::Loaded(models::album(row));
                }
            }
            Read::Playlist(id) => {
                if let Some(row) = rows.first() {
                    self.playlist_pages.entry(id).or_default().playlist =
                        Loadable::Loaded(models::playlist(row));
                }
            }
            Read::Artist(id) => {
                if let Some(row) = rows.first() {
                    self.artist_pages.entry(id).or_default().artist =
                        Loadable::Loaded(models::artist(row));
                }
            }
            Read::AlbumTracks(id) => {
                let tracks = self.apple_tracks(data);
                self.album_pages
                    .entry(id)
                    .or_default()
                    .tracks
                    .absorb(offset, page(tracks, data, offset));
            }
            Read::PlaylistTracks(id) => {
                let tracks = self.apple_tracks(data);
                self.playlist_pages.entry(id).or_default().items.absorb(
                    offset,
                    page(
                        tracks
                            .into_iter()
                            .map(|track| PlaylistItem {
                                item: Some(PlayableItem::Track(track)),
                                ..Default::default()
                            })
                            .collect(),
                        data,
                        offset,
                    ),
                );
            }
            Read::ArtistAlbums(id) => {
                self.artist_pages
                    .entry(id.clone())
                    .or_default()
                    .albums
                    .entry("album,single,compilation".into())
                    .or_default()
                    .absorb(
                        offset,
                        page(rows.iter().map(models::album).collect(), data, offset),
                    );
                self.apple_artist_filters(&id);
            }
            Read::ArtistSongs(id) => {
                let tracks = self.apple_tracks(data);
                self.artist_pages.entry(id).or_default().top_tracks = Loadable::Loaded(tracks);
            }
            Read::Search { .. } => {}
        }
    }
    fn apple_tracks(&mut self, data: &Value) -> Vec<Track> {
        resources(data)
            .iter()
            .filter_map(models::song)
            .map(|song| {
                let track = song.track();
                if let Some(id) = &track.id {
                    self.track_cache.insert(id.clone(), track.clone());
                }
                self.apple
                    .as_mut()
                    .unwrap()
                    .known_songs
                    .insert(song.uri(), song);
                track
            })
            .collect()
    }
}
fn resources(data: &Value) -> Vec<Value> {
    data["data"].as_array().cloned().unwrap_or_default()
}
pub(super) fn read_for_page(target: &Read, page: &Page) -> bool {
    match (page, target) {
        (Page::Album(id), Read::Album(held) | Read::AlbumTracks(held))
        | (Page::Playlist(id), Read::Playlist(held) | Read::PlaylistTracks(held))
        | (
            Page::Artist(id),
            Read::Artist(held) | Read::ArtistAlbums(held) | Read::ArtistSongs(held),
        ) => id == held,
        _ => false,
    }
}
fn merge<T: Default>(held: &mut Option<ApiPage<T>>, fresh: Option<ApiPage<T>>) {
    if let Some(fresh) = fresh {
        let held = held.get_or_insert_with(ApiPage::default);
        held.items.extend(fresh.items);
        held.total = held.items.len() as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn mini_player_queue_actions_preserve_occurrences_and_album_adds_are_atomic() {
        let mut app = super::super::tests::test_app("apple-mini-queue");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        state.loading = false;
        app.apple = Some(state);
        let songs = ["i.a", "i.b", "i.c"].iter().map(|id| models::song(&json!({
            "id":id,"type":"library-songs","attributes":{"name":id,"playParams":{"id":id,"kind":"song","isLibrary":true}},
        })).unwrap()).collect::<Vec<_>>();
        app.apple.as_mut().unwrap().songs = songs;
        let uris = app
            .apple
            .as_ref()
            .unwrap()
            .songs
            .iter()
            .map(crate::apple::Song::uri)
            .collect::<Vec<_>>();
        let ctx = egui::Context::default();
        app.apply(
            Action::PlayUris {
                uris: uris.clone(),
                index: 0,
            },
            &ctx,
        );
        app.apply(
            Action::QueueMany {
                songs: vec![
                    (uris[1].clone(), "B".into()),
                    (uris[2].clone(), "C".into()),
                    (uris[1].clone(), "B".into()),
                ],
            },
            &ctx,
        );
        assert_eq!(app.queued_rows_len(), 3);
        app.apply(
            Action::PlayFromRow {
                context: RowContext::Queue,
                uri: uris[1].clone(),
                index: 2,
            },
            &ctx,
        );
        assert_eq!(app.apple.as_ref().unwrap().index, Some(5));
        assert_eq!(app.queued_rows_len(), 0);
        let remaining = app
            .queue
            .get()
            .unwrap()
            .queue
            .iter()
            .map(|item| item.uri().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(remaining, uris[1..]);
        assert_eq!(app.local.track.as_ref().unwrap().uri, uris[1]);
        app.apply(
            Action::AddToQueue {
                uri: "apple:album:library.l.album".into(),
                label: "Album".into(),
            },
            &ctx,
        );
        assert!(!app.pending_album_queues.is_empty());
        let read = app
            .apple
            .as_ref()
            .unwrap()
            .reads
            .iter()
            .find(|(_, (target, _))| matches!(target, Read::AlbumTracks(_)))
            .map(|(id, _)| *id)
            .unwrap();
        let resource = json!({"id":"i.a","type":"library-songs","attributes":{"name":"A","playParams":{"id":"i.a","kind":"song","isLibrary":true}}});
        app.apple_response(&json!({"id":read,"data":{"data":[resource.clone()],"next":"/v1/me/library/albums/l.album/tracks?offset=1"}}));
        app.apple_finish_album_queues();
        assert_eq!(app.queued_rows_len(), 0);
        let read = app
            .apple
            .as_ref()
            .unwrap()
            .reads
            .iter()
            .find(|(_, (target, _))| matches!(target, Read::AlbumTracks(_)))
            .map(|(id, _)| *id)
            .unwrap();
        app.apple_response(&json!({"id":read,"error":"Unavailable"}));
        app.apple_finish_album_queues();
        assert!(app.pending_album_queues.is_empty());
        assert_eq!(app.queued_rows_len(), 0);
        app.apply(Action::ClearQueue, &ctx);
        app.apply(
            Action::AddToQueue {
                uri: "apple:album:library.l.album".into(),
                label: "Album".into(),
            },
            &ctx,
        );
        app.apple_load_more(Page::Album("library.l.album".into()));
        let read = app
            .apple
            .as_ref()
            .unwrap()
            .reads
            .iter()
            .find(|(_, (target, _))| matches!(target, Read::AlbumTracks(_)))
            .map(|(id, _)| *id)
            .unwrap();
        app.apply(Action::ClearQueue, &ctx);
        app.apple_response(&json!({"id":read,"data":{"data":[resource]}}));
        app.apple_finish_album_queues();
        assert_eq!(app.queued_rows_len(), 0); // Clear cancels a late album completion.
    }
    #[test]
    fn apple_pages_preserve_pagination_and_stale_search_and_authorization() {
        let mut app = super::super::tests::test_app("apple-page-routing");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        state.loading = false;
        app.apple = Some(state);
        let read = app.apple.as_mut().unwrap().read(
            Read::Albums,
            "/v1/me/library/albums?limit=100".into(),
            0,
        );
        let resource =
            json!({"id":"l.upload","type":"library-albums","attributes":{"name":"Uploaded album"}});
        app.apple_response(&json!({"id":read["id"],"data":{"data":[resource],"next":"/v1/me/library/albums?offset=1"}}));
        assert_eq!(
            app.library.albums.items[0].album.uri,
            "apple:album:library.l.upload"
        );
        assert_eq!(app.library.albums.next_offset, Some(1));
        assert_eq!(app.apple.as_ref().unwrap().next_reads[&Read::Albums].1, 1);
        app.search.serial = 2;
        let read = app.apple.as_mut().unwrap().read(
            Read::Search {
                serial: 1,
                library: true,
            },
            "/v1/me/library/search?term=old".into(),
            0,
        );
        app.apple_response(&json!({"id":read["id"],"data":{"results":{}}}));
        assert!(app.search.results.get().is_none());
        let song=models::song(&json!({"id":"123","type":"songs","attributes":{"name":"Cached song","playParams":{"id":"123","kind":"song"}}})).unwrap();
        app.apple
            .as_mut()
            .unwrap()
            .known_songs
            .insert(song.uri(), song.clone());
        app.apple_evict_songs();
        assert!(app.apple.as_ref().unwrap().known_songs.is_empty());
        let track = song.track();
        app.track_cache.insert(track.id.clone().unwrap(), track);
        app.apple
            .as_mut()
            .unwrap()
            .known_songs
            .insert(song.uri(), song);
        app.apple_evict_songs();
        assert_eq!(app.apple.as_ref().unwrap().known_songs.len(), 1);
        app.offline = false;
        app.handle_backend_events(vec![Event::Auth(AuthStatus::SignedOut)]);
        assert!(app.account_ready());
        app.handle_backend_events(vec![Event::Apple {
            generation: 0,
            value: json!({"type":"library","session":1,"items":[],"next":null}),
        }]);
        assert!(!app.library.liked.loaded_once);
        app.apply(Action::SignOut, &egui::Context::default());
        assert!(app.library.albums.items.is_empty());
        app.handle_backend_events(vec![Event::Apple {
            generation: 1,
            value: json!({"type":"authorized","session":1}),
        }]);
        assert!(!app.account_ready());
    }
}
