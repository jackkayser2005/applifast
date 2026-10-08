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
    pub(super) fn apple_album_to_playlist(
        &mut self,
        uri: &str,
        label: &str,
        playlist: Option<(String, String)>,
    ) {
        if self.playlist_busy || !self.account_ready() {
            return;
        }
        let Some(Page::Album(id)) = Page::from_uri(uri) else {
            return;
        };
        if self.apple_resource_path("albums", &id).is_none() {
            self.toast_error("Apple Music could not resolve this album. Reload it and retry.");
            return;
        }
        if playlist
            .as_ref()
            .is_some_and(|(id, _)| !self.apple.as_ref().unwrap().writable_playlists.contains(id))
        {
            self.toast_error("Refresh your library and choose a writable Apple playlist.");
            return;
        }
        self.apple.as_mut().unwrap().pending_album_playlist = Some(Action::AddAlbumToPlaylist {
            uri: uri.into(),
            label: label.into(),
            playlist,
        });
        self.playlist_busy = true;
        self.apple_ensure_loaded(Page::Album(id.clone()));
        if self
            .album_pages
            .get(&id)
            .is_some_and(|page| page.tracks.loading)
        {
            self.toast("Loading every album track before adding it to a playlist.");
        }
        self.apple_finish_album_playlist();
    }
    pub(super) fn apple_finish_album_playlist(&mut self) {
        let Some(Action::AddAlbumToPlaylist {
            uri,
            label,
            playlist,
        }) = self
            .apple
            .as_ref()
            .and_then(|apple| apple.pending_album_playlist.clone())
        else {
            return;
        };
        let Some(Page::Album(id)) = Page::from_uri(&uri) else {
            return;
        };
        let Some(page) = self.album_pages.get(&id) else {
            return;
        };
        if page.tracks.loading {
            return;
        }
        if let Some(error) = &page.tracks.error {
            let error = error.clone();
            self.apple.as_mut().unwrap().pending_album_playlist = None;
            self.playlist_busy = false;
            self.toast_error(error);
        } else if page.tracks.is_complete() {
            let items = page
                .tracks
                .items
                .iter()
                .cloned()
                .map(PlayableItem::Track)
                .collect::<Vec<_>>();
            self.apple.as_mut().unwrap().pending_album_playlist = None;
            self.playlist_busy = false;
            if items.is_empty() {
                self.toast_error("Apple returned no songs for this album.");
                return;
            }
            if let Some((id, name)) = playlist {
                self.apple_add_to_playlist(&id, &name, &items, true);
            } else {
                self.dialog = Some(Dialog::CreatePlaylist {
                    name: label,
                    public: false,
                    add_uris: items.iter().map(|item| item.uri().to_owned()).collect(),
                });
            }
        } else {
            self.apple_load_more(Page::Album(id));
        }
    }
    pub(super) fn apple_add_to_playlist(
        &mut self,
        id: &str,
        name: &str,
        items: &[PlayableItem],
        check_duplicates: bool,
    ) {
        if self.playlist_busy {
            return;
        }
        if !self.account_ready() || !self.apple.as_ref().unwrap().writable_playlists.contains(id) {
            self.toast_error("Apple Music has not marked this playlist writable. Refresh your library and retry.");
            return;
        }
        if self
            .apple
            .as_ref()
            .unwrap()
            .playlist_confirms
            .contains_key(id)
        {
            self.toast_error(
                "Wait for Apple to confirm this playlist, or refresh it before adding more songs.",
            );
            return;
        }
        let uris = items
            .iter()
            .map(|item| item.uri().to_owned())
            .collect::<Vec<_>>();
        let command = applifast_playback_probe::protocol::Command::AppendPlaylist {
            id: 0,
            playlist: id.strip_prefix("library.").unwrap_or("").into(),
            items: match self.apple.as_ref().unwrap().playlist_items(&uris) {
                Ok(items) => items,
                Err(error) => {
                    self.toast_error(error);
                    return;
                }
            },
        };
        if let Err(error) = command.validate() {
            self.toast_error(error);
            return;
        }
        if !self
            .playlist_pages
            .get(id)
            .is_some_and(|page| page.items.is_complete())
        {
            // ponytail: load the destination for exact duplicate/occurrence checks.
            // Stream these checks if large playlists cause measured memory pressure.
            self.apple.as_mut().unwrap().pending_playlist_add = Some(if check_duplicates {
                Action::AddToPlaylist {
                    playlist_id: id.into(),
                    playlist_name: name.into(),
                    items: items.to_vec(),
                }
            } else {
                Action::ConfirmAddToPlaylist {
                    playlist_id: id.into(),
                    playlist_name: name.into(),
                    items: items.to_vec(),
                    position: None,
                }
            });
            self.playlist_busy = true;
            self.toast("Loading the destination playlist before adding songs.");
            self.apple_ensure_loaded(Page::Playlist(id.into()));
            self.apple_finish_playlist_add();
            return;
        }
        if check_duplicates {
            let duplicate_uris = self.local_playlist_duplicates(id, items).unwrap();
            if !duplicate_uris.is_empty() {
                self.dialog = Some(Dialog::ConfirmPlaylistDuplicates {
                    playlist_id: id.into(),
                    playlist_name: name.into(),
                    items: items.to_vec(),
                    position: None,
                    duplicate_uris,
                });
                return;
            }
        }
        let before = self.playlist_pages[id].items.items.len();
        let command = match self
            .apple
            .as_mut()
            .unwrap()
            .append_playlist(id, &uris, before)
        {
            Ok(command) => command,
            Err(error) => {
                self.toast_error(error);
                return;
            }
        };
        let apple = self.apple.as_mut().unwrap();
        // Requests issued before the edit describe the old rows and cannot confirm a write.
        apple.reads.retain(|_, (target, _)| {
            !matches!(target,
            Read::Playlist(held) | Read::PlaylistTracks(held) if held == id)
        });
        apple.next_reads.remove(&Read::PlaylistTracks(id.into()));
        let page = self.playlist_pages.get_mut(id).unwrap();
        page.pending_writes = 1;
        page.snapshot_rechecks = 0;
        page.items
            .items
            .extend(items.iter().cloned().map(|item| PlaylistItem {
                item: Some(item),
                ..Default::default()
            }));
        page.items.total = Some(page.items.items.len() as u32);
        page.items.revision = page.items.revision.wrapping_add(1);
        self.apple_set_playlist_total(id);
        self.playlist_busy = true;
        self.dialog = None;
        self.backend.send(Command::AppleSend(command.to_string()));
    }
    pub(super) fn apple_finish_playlist_add(&mut self) {
        let Some(action) = self
            .apple
            .as_ref()
            .and_then(|apple| apple.pending_playlist_add.clone())
        else {
            return;
        };
        let (id, name, items, check) = match &action {
            Action::AddToPlaylist {
                playlist_id,
                playlist_name,
                items,
            } => (playlist_id, playlist_name, items, true),
            Action::ConfirmAddToPlaylist {
                playlist_id,
                playlist_name,
                items,
                ..
            } => (playlist_id, playlist_name, items, false),
            _ => return,
        };
        let Some(page) = self.playlist_pages.get(id) else {
            return;
        };
        if page.items.loading {
            return;
        }
        if let Some(error) = &page.items.error {
            let error = error.clone();
            self.apple.as_mut().unwrap().pending_playlist_add = None;
            self.playlist_busy = false;
            self.toast_error(error);
        } else if page.items.is_complete() {
            self.apple.as_mut().unwrap().pending_playlist_add = None;
            self.playlist_busy = false;
            self.apple_add_to_playlist(id, name, items, check);
        } else {
            self.apple_load_more(Page::Playlist(id.clone()));
        }
    }
    fn apple_set_playlist_total(&mut self, id: &str) {
        let total = self.playlist_pages[id].items.items.len() as u32;
        for playlist in self
            .playlist_pages
            .get_mut(id)
            .unwrap()
            .playlist
            .get_mut()
            .into_iter()
            .chain(
                self.library
                    .playlists
                    .get_mut()
                    .into_iter()
                    .flatten()
                    .filter(|row| row.id == id),
            )
        {
            playlist.tracks = Some(crate::api::models::TrackCount { total });
        }
    }
    fn apple_playlist_appended(&mut self, id: String, before: usize, value: &Value) {
        self.playlist_busy = false;
        let Some(page) = self.playlist_pages.get_mut(&id) else {
            return;
        };
        page.pending_writes = 0;
        if !value["error"].is_null() {
            page.items.items.truncate(before);
            page.items.total = Some(before as u32);
            page.items.revision = page.items.revision.wrapping_add(1);
            self.apple_set_playlist_total(&id);
            self.toast_error("Apple Music could not confirm the added songs. Check the playlist before retrying to avoid duplicates.");
        } else {
            self.apple
                .as_mut()
                .unwrap()
                .playlist_confirms
                .insert(id, Default::default());
            self.toast("Songs added. Waiting for Apple to show the updated playlist.");
        }
        self.apple.as_mut().unwrap().playlist_recheck_at =
            Some(Instant::now() + Duration::from_secs(2));
    }
    pub(super) fn apple_create_playlist(&mut self, name: &str, public: bool, uris: &[String]) {
        if self.playlist_busy {
            return;
        }
        if !self.account_ready() {
            self.toast_error("Sign in to Apple Music before creating a playlist.");
            return;
        }
        let result = self
            .apple
            .as_mut()
            .unwrap()
            .create_playlist(name, public, uris);
        let (id, command) = match result {
            Ok(value) => value,
            Err(error) => {
                self.toast_error(error);
                return;
            }
        };
        let tracks = uris
            .iter()
            .map(|uri| PlaylistItem {
                item: Some(PlayableItem::Track(
                    self.apple.as_ref().unwrap().find_song(uri).unwrap().track(),
                )),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let playlist = Playlist {
            id: id.clone(),
            uri: format!("apple:playlist:{id}"),
            name: name.trim().into(),
            public: Some(public),
            owner: crate::api::models::Owner {
                display_name: Some("Apple Music".into()),
                ..Default::default()
            },
            tracks: Some(crate::api::models::TrackCount {
                total: tracks.len() as u32,
            }),
            ..Default::default()
        };
        let mut entry = PlaylistPage {
            playlist: Loadable::Loaded(playlist.clone()),
            pending_writes: 1,
            ..Default::default()
        };
        entry
            .items
            .absorb(0, page(tracks, &serde_json::json!({}), 0));
        self.playlist_pages.insert(id, entry);
        if self.library.playlists.get().is_none() {
            self.library.playlists = Loadable::Loaded(Vec::new());
        }
        self.library
            .playlists
            .get_mut()
            .unwrap()
            .insert(0, playlist);
        self.dialog = Some(Dialog::CreatePlaylist {
            name: name.into(),
            public,
            add_uris: uris.to_vec(),
        });
        self.playlist_busy = true;
        self.backend.send(Command::AppleSend(command.to_string()));
    }

    fn apple_playlist_created(&mut self, temporary: String, value: &Value) {
        self.playlist_busy = false;
        let Some(mut entry) = self.playlist_pages.remove(&temporary) else {
            return;
        };
        let created = resources(&value["data"]).into_iter().find(|row| {
            row["type"] == "library-playlists"
                && row["id"]
                    .as_str()
                    .is_some_and(applifast_playback_probe::protocol::valid_library_playlist_id)
        });
        let Some(resource) = created.filter(|_| value["error"].is_null()) else {
            if let Some(rows) = self.library.playlists.get_mut() {
                rows.retain(|row| row.id != temporary);
            }
            for page in &mut self.history {
                if *page == Page::Playlist(temporary.clone()) {
                    *page = Page::Home;
                }
            }
            if self
                .assumed_context
                .as_ref()
                .is_some_and(|held| held.uri == format!("apple:playlist:{temporary}"))
            {
                self.assumed_context = None;
            }
            if self
                .selection
                .as_ref()
                .is_some_and(|(page, _, _)| *page == Page::Playlist(temporary.clone()))
            {
                self.selection = None;
            }
            self.table_rows.remove(&Page::Playlist(temporary));
            self.toast_error("Apple Music could not confirm the new playlist. Check your library before retrying to avoid creating a duplicate.");
            self.apple.as_mut().unwrap().playlist_recheck_at =
                Some(Instant::now() + Duration::from_secs(2));
            return;
        };
        let mut playlist = models::playlist(&resource);
        self.apple
            .as_mut()
            .unwrap()
            .remember_playlist_permissions(std::slice::from_ref(&resource));
        let submitted = entry.playlist.get().unwrap();
        let close_dialog = matches!(&self.dialog, Some(Dialog::CreatePlaylist { name, public, add_uris })
            if name.trim() == submitted.name && Some(*public) == submitted.public
                && *add_uris == entry.items.items.iter().filter_map(|item| item.playable().map(|item| item.uri().to_owned())).collect::<Vec<_>>());
        // Creation already accepted these occurrences. A lagging read must not erase them.
        playlist.tracks = submitted.tracks.clone();
        entry.playlist = Loadable::Loaded(playlist.clone());
        entry.pending_writes = 0;
        if let Some(rows) = self.library.playlists.get_mut() {
            rows.retain(|row| row.id != playlist.id);
            if let Some(row) = rows.iter_mut().find(|row| row.id == temporary) {
                *row = playlist.clone();
            } else {
                rows.insert(0, playlist.clone());
            }
        }
        for page in &mut self.history {
            if *page == Page::Playlist(temporary.clone()) {
                *page = Page::Playlist(playlist.id.clone());
            }
        }
        if let Some(held) = &mut self.assumed_context
            && held.uri == format!("apple:playlist:{temporary}")
        {
            held.uri = playlist.uri.clone();
        }
        if let Some((page, _, _)) = &mut self.selection
            && *page == Page::Playlist(temporary.clone())
        {
            *page = Page::Playlist(playlist.id.clone());
        }
        self.table_rows.remove(&Page::Playlist(temporary));
        self.playlist_pages.insert(playlist.id.clone(), entry);
        let apple = self.apple.as_mut().unwrap();
        apple.playlist_cards.insert(playlist.id.clone());
        apple
            .playlist_confirms
            .insert(playlist.id.clone(), Default::default());
        apple.playlist_recheck_at = Some(Instant::now() + Duration::from_secs(2));
        self.toast(gettext(self.locale, "Created {name}").replace("{name}", &playlist.name));
        if close_dialog {
            self.dialog = None;
            self.open(Page::Playlist(playlist.id));
        }
    }

    pub(super) fn apple_recheck_playlists(&mut self, ctx: &egui::Context, now: Instant) {
        let Some(apple) = &mut self.apple else {
            return;
        };
        let Some(due) = apple.playlist_recheck_at else {
            return;
        };
        if now < due {
            ctx.request_repaint_after(due - now);
            return;
        }
        apple.playlist_recheck_at = None;
        let ids = apple.playlist_confirms.keys().cloned().collect::<Vec<_>>();
        self.apple_read(
            Read::Playlists,
            "/v1/me/library/playlists?limit=100".into(),
            0,
        );
        for id in ids {
            if self
                .apple
                .as_ref()
                .unwrap()
                .reads
                .values()
                .any(|(target, _)| *target == Read::PlaylistTracks(id.clone()))
            {
                continue;
            }
            let Some(path) = self.apple_resource_path("playlists", &id) else {
                continue;
            };
            let Some(entry) = self.playlist_pages.get_mut(&id) else {
                continue;
            };
            if entry.snapshot_rechecks >= 3 {
                continue;
            }
            entry.snapshot_rechecks += 1;
            self.apple
                .as_mut()
                .unwrap()
                .playlist_confirms
                .insert(id.clone(), Default::default());
            self.apple_read(
                Read::PlaylistTracks(id),
                format!("{path}/tracks?limit=100"),
                0,
            );
        }
    }

    fn apple_confirm_playlist(&mut self, id: &str) {
        let apple = self.apple.as_ref().unwrap();
        let fresh = &apple.playlist_confirms[id];
        if !fresh.is_complete() {
            return;
        }
        let entry = &self.playlist_pages[id];
        let matches = fresh.items.len() >= entry.items.items.len()
            && entry
                .items
                .items
                .iter()
                .zip(&fresh.items)
                .all(|(expected, actual)| {
                    let Some(expected) = expected
                        .playable()
                        .and_then(|item| apple.find_song(item.uri()))
                    else {
                        return false;
                    };
                    let Some(actual) = actual
                        .playable()
                        .and_then(|item| apple.find_song(item.uri()))
                    else {
                        return false;
                    };
                    expected.item.kind == actual.item.kind && expected.item.id == actual.item.id
                        || expected.item.kind
                            == applifast_playback_probe::protocol::ItemKind::Catalog
                            && actual.catalog_id.as_deref() == Some(expected.item.id.as_str())
                });
        if matches {
            let fresh = self
                .apple
                .as_mut()
                .unwrap()
                .playlist_confirms
                .remove(id)
                .unwrap();
            self.playlist_pages.get_mut(id).unwrap().items = fresh;
            self.apple_set_playlist_total(id);
        } else {
            self.apple_retry_playlist_confirmation(id);
        }
    }

    fn apple_retry_playlist_confirmation(&mut self, id: &str) {
        self.apple
            .as_mut()
            .unwrap()
            .next_reads
            .remove(&Read::PlaylistTracks(id.into()));
        if self.playlist_pages[id].snapshot_rechecks < 3 {
            self.apple.as_mut().unwrap().playlist_recheck_at =
                Some(Instant::now() + Duration::from_secs(2));
        } else {
            self.playlist_pages.get_mut(id).unwrap().items.error =
                Some("Apple has not shown the saved tracks yet. Refresh to check again.".into());
        }
    }
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
        if let Some(
            Action::AddToPlaylist { items, .. } | Action::ConfirmAddToPlaylist { items, .. },
        ) = &apple.pending_playlist_add
        {
            needed.extend(items.iter().map(|item| item.uri().to_owned()));
        }
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
    pub(super) fn apple_read(&mut self, target: Read, path: String, offset: u32) {
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
            Page::Home => {
                for shelf in crate::apple::HomeShelf::ALL {
                    if shelf == crate::apple::HomeShelf::Recommendations
                        && !self.settings.home.recommendations.visible
                    {
                        continue;
                    }
                    if self.apple.as_ref().unwrap().home.contains_key(&shelf) {
                        continue;
                    }
                    self.apple
                        .as_mut()
                        .unwrap()
                        .home
                        .insert(shelf, Loadable::Loading);
                    self.apple_read(Read::Home(shelf), shelf.path().into(), 0);
                }
            }
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
                let listed = self.library_entry(&id).cloned();
                let entry = self.playlist_pages.entry(id.clone()).or_default();
                entry.playlist = listed.map_or(Loadable::Loading, Loadable::Loaded);
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
        if page == Page::Home {
            for shelf in crate::apple::HomeShelf::ALL {
                let target = Read::Home(shelf);
                if self
                    .apple
                    .as_ref()
                    .unwrap()
                    .reads
                    .values()
                    .any(|(read, _)| *read == target)
                {
                    continue;
                }
                if let Some((path, offset)) = self
                    .apple
                    .as_ref()
                    .unwrap()
                    .next_reads
                    .get(&target)
                    .cloned()
                {
                    self.apple_read(target, path, offset);
                }
            }
            return;
        }
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
    pub(crate) fn apple_response(&mut self, value: &Value) {
        if let Some((id, before)) = value["id"]
            .as_u64()
            .and_then(|id| self.apple.as_mut()?.playlist_appends.remove(&id))
        {
            self.apple_playlist_appended(id, before, value);
            return;
        }
        if let Some(temporary) = value["id"]
            .as_u64()
            .and_then(|id| self.apple.as_mut()?.playlist_creates.remove(&id))
        {
            self.apple_playlist_created(temporary, value);
            return;
        }
        let Some((target, offset)) = value["id"]
            .as_u64()
            .and_then(|id| self.apple.as_mut()?.reads.remove(&id))
        else {
            return;
        };
        if let Read::Home(shelf) = target {
            if let Some(error) = value["error"].as_str() {
                // A failed later page keeps the playable cards already shown.
                if self
                    .apple
                    .as_ref()
                    .unwrap()
                    .home
                    .get(&shelf)
                    .and_then(Loadable::get)
                    .is_some()
                {
                    self.toast_error(error);
                } else {
                    self.apple
                        .as_mut()
                        .unwrap()
                        .home
                        .insert(shelf, Loadable::Failed(error.into()));
                }
                return;
            }
            let data = &value["data"];
            if !data["data"].is_array() {
                let error = "Apple Music returned an incomplete shelf. Retry.";
                if self
                    .apple
                    .as_ref()
                    .unwrap()
                    .home
                    .get(&shelf)
                    .and_then(Loadable::get)
                    .is_some()
                {
                    self.toast_error(error);
                } else {
                    self.apple
                        .as_mut()
                        .unwrap()
                        .home
                        .insert(shelf, Loadable::Failed(error.into()));
                }
                return;
            }
            let raw = resources(data);
            let rows = if shelf == crate::apple::HomeShelf::Recommendations {
                raw.iter()
                    .flat_map(|row| resources(&row["relationships"]["contents"]))
                    .collect::<Vec<_>>()
            } else {
                raw.clone()
            };
            self.apple_tracks(&serde_json::json!({"data":rows}));
            let cards = rows
                .iter()
                .filter_map(models::home_card)
                .collect::<Vec<_>>();
            let apple = self.apple.as_mut().unwrap();
            let held = apple.home.entry(shelf).or_default();
            if offset == 0 || held.get().is_none() {
                *held = Loadable::Loaded(Vec::new());
            }
            let held = held.get_mut().unwrap();
            for card in cards {
                if held.len() < 64 && !held.iter().any(|row| row.uri == card.uri) {
                    held.push(card);
                }
            }
            let target = Read::Home(shelf);
            let previous = apple.next_reads.remove(&target);
            if held.len() < 64
                && !raw.is_empty()
                && let Some(next) = data["next"].as_str().filter(|next| {
                    applifast_playback_probe::protocol::valid_read_path(next)
                        && next.split('?').next() == shelf.path().split('?').next()
                        && previous.as_ref().is_none_or(|(path, _)| path != next)
                })
            {
                apple
                    .next_reads
                    .insert(target, (next.into(), offset + raw.len() as u32));
            }
            return;
        }
        if let Some(error) = value["error"].as_str() {
            match &target {
                Read::Playlists if self.library.playlists.get().is_none() => {
                    self.library.playlists = Loadable::Failed(error.into())
                }
                Read::Playlists => self.toast_error(error),
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
                    if self
                        .apple
                        .as_ref()
                        .unwrap()
                        .playlist_confirms
                        .contains_key(id)
                    {
                        self.apple_retry_playlist_confirmation(id);
                        return;
                    }
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
        self.apple
            .as_mut()
            .unwrap()
            .remember_playlist_permissions(&rows);
        let repeated_next = offset > 0
            && self
                .apple
                .as_ref()
                .unwrap()
                .next_reads
                .get(&target)
                .is_some_and(|(path, _)| data["next"].as_str() == Some(path));
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
            Read::Home(_) => unreachable!("home responses are handled above"),
            Read::Playlists => {
                let mut playlists = rows.iter().map(models::playlist).collect::<Vec<_>>();
                self.apple
                    .as_mut()
                    .unwrap()
                    .playlist_cards
                    .retain(|id| !playlists.iter().any(|row| row.id == *id));
                if offset == 0 {
                    let apple = self.apple.as_ref().unwrap();
                    let ids = apple
                        .playlist_creates
                        .values()
                        .chain(apple.playlist_appends.values().map(|(id, _)| id))
                        .chain(apple.playlist_confirms.keys())
                        .chain(&apple.playlist_cards)
                        .collect::<HashSet<_>>();
                    let held = self
                        .library
                        .playlists
                        .get()
                        .into_iter()
                        .flatten()
                        .filter(|row| ids.contains(&row.id))
                        .cloned()
                        .collect::<Vec<_>>();
                    playlists.retain(|row| !held.iter().any(|held| held.id == row.id));
                    playlists.splice(0..0, held);
                    self.library.playlists = Loadable::Loaded(playlists);
                } else if let Some(held) = self.library.playlists.get_mut() {
                    for row in playlists {
                        if !held.iter().any(|held| held.id == row.id) {
                            held.push(row);
                        }
                    }
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
                            added_at: models::added_at(row),
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
                if matches!(&self.apple.as_ref().unwrap().pending_album_playlist,
                    Some(Action::AddAlbumToPlaylist { uri, .. }) if Page::from_uri(uri) == Some(Page::Album(id.clone())))
                    && (repeated_next
                        || rows.iter().any(|row| models::song(row).is_none())
                        || data["data"]
                            .as_array()
                            .is_none_or(|rows| rows.is_empty() && data["next"].is_string())
                        || data
                            .get("next")
                            .filter(|value| !value.is_null())
                            .is_some_and(|value| {
                                value.as_str().is_none_or(|next| {
                                    !applifast_playback_probe::protocol::valid_read_path(next)
                                        || !self.apple_resource_path("albums", &id).is_some_and(
                                            |path| next.starts_with(&format!("{path}/tracks?")),
                                        )
                                })
                            }))
                {
                    let page = self.album_pages.get_mut(&id).unwrap();
                    page.tracks.loading = false;
                    page.tracks.error = Some(
                        "Apple returned an invalid album page. Refresh the album and retry.".into(),
                    );
                    self.apple
                        .as_mut()
                        .unwrap()
                        .next_reads
                        .remove(&Read::AlbumTracks(id));
                    return;
                }
                let tracks = self.apple_tracks(data);
                self.album_pages
                    .entry(id)
                    .or_default()
                    .tracks
                    .absorb(offset, page(tracks, data, offset));
            }
            Read::PlaylistTracks(id) => {
                let confirming = self
                    .apple
                    .as_ref()
                    .unwrap()
                    .playlist_confirms
                    .contains_key(&id);
                let preparing = matches!(&self.apple.as_ref().unwrap().pending_playlist_add,
                    Some(Action::AddToPlaylist { playlist_id, .. } | Action::ConfirmAddToPlaylist { playlist_id, .. }) if playlist_id == &id);
                if confirming || preparing {
                    let prefix = self
                        .apple_resource_path("playlists", &id)
                        .map(|path| format!("{path}/tracks?"));
                    let next = data.get("next").filter(|value| !value.is_null());
                    if repeated_next
                        || !data["data"].is_array()
                        || next.is_some_and(|value| {
                            value.as_str().is_none_or(|path| {
                                !applifast_playback_probe::protocol::valid_read_path(path)
                                    || prefix
                                        .as_ref()
                                        .is_none_or(|prefix| !path.starts_with(prefix))
                            })
                        })
                        || next.is_some() && rows.is_empty()
                    {
                        if confirming {
                            self.apple_retry_playlist_confirmation(&id);
                        } else {
                            let entry = self.playlist_pages.get_mut(&id).unwrap();
                            entry.items.loading = false;
                            entry.items.error = Some("Apple returned an invalid playlist page. Refresh the playlist and retry adding songs.".into());
                            self.apple
                                .as_mut()
                                .unwrap()
                                .next_reads
                                .remove(&Read::PlaylistTracks(id));
                        }
                        return;
                    }
                }
                let tracks = self.apple_tracks(data);
                if let Some(held) = self.apple.as_mut().unwrap().playlist_confirms.get_mut(&id) {
                    held.absorb(
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
                    if let Some((path, offset)) = self
                        .apple
                        .as_ref()
                        .unwrap()
                        .next_reads
                        .get(&Read::PlaylistTracks(id.clone()))
                        .cloned()
                    {
                        self.apple_read(Read::PlaylistTracks(id.clone()), path, offset);
                    } else {
                        self.apple_confirm_playlist(&id);
                    }
                    return;
                }
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
        (Page::Home, Read::Home(_)) => true,
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

    #[cfg(windows)]
    #[test]
    #[ignore = "Reads Home and album dates using the locally authorized Apple account"]
    fn native_host_reads_home_and_album_dates() {
        let (sender, events) = std::sync::mpsc::channel();
        let host = crate::player::AppleHost::start(None, move |event| {
            let _ = sender.send(event);
        });
        let mut app = super::super::tests::test_app("apple-home-native");
        app.backend.set_offline(true);
        let check = || {
            let await_event = |kind: &str, id: Option<u64>| {
                let deadline = Instant::now() + Duration::from_secs(45);
                loop {
                    let event = events
                        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                        .unwrap_or_else(|_| panic!("Host timed out awaiting {kind}"));
                    if event["type"] == "error" {
                        // Fixed diagnoses only; never log authorization or SDK error text.
                        let diagnosis = match event["message"].as_str() {
                            Some("Import your developer token with --token-file PATH.") => {
                                "missing developer token"
                            }
                            Some("Developer token expired. Generate and import a fresh token.") => {
                                "expired developer token"
                            }
                            Some(
                                "Expected a signed MusicKit developer JWT, not a .p8 signing key.",
                            ) => "invalid developer token format",
                            Some(
                                "Developer token dates are invalid. Check the signing machine's clock.",
                            ) => "invalid token dates",
                            Some(
                                "Developer token's origin must permit https://applifast.invalid.",
                            ) => "invalid token origin",
                            Some("System clock is before the Unix epoch.") => {
                                "invalid system clock"
                            }
                            Some("LOCALAPPDATA is unavailable.") => {
                                "missing local app data directory"
                            }
                            Some("Cannot read sign-out state.") => "cannot read sign-out marker",
                            Some("Stored credential is invalid.") => "invalid stored credential",
                            Some(
                                "Windows Credential Manager is unavailable."
                                | "Cannot read Windows Credential Manager.",
                            ) => "credential store unavailable",
                            Some(
                                "Probe did not finish successfully."
                                | "Playback host thread stopped unexpectedly.",
                            ) => "native host stopped",
                            _ => "unclassified host initialization failure",
                        };
                        panic!("{diagnosis}");
                    }
                    if event["type"] == kind && id.is_none_or(|id| event["id"] == id) {
                        return event;
                    }
                }
            };
            let ready = await_event("ready", None);
            assert_eq!(ready["authorized"], true, "Authorize the local host first");
            let mut state = crate::apple::State::default();
            state.authorized = true;
            state.ready = true;
            app.apple = Some(state);
            for shelf in crate::apple::HomeShelf::ALL {
                let command =
                    app.apple
                        .as_mut()
                        .unwrap()
                        .read(Read::Home(shelf), shelf.path().into(), 0);
                host.send(command.to_string());
                let response = await_event("response", command["id"].as_u64());
                assert!(response["error"].is_null(), "Home {shelf:?} read failed");
                app.apple_response(&response);
                let cards = app.apple.as_ref().unwrap().home[&shelf].get().unwrap();
                eprintln!("Home {shelf:?}: {} loaded cards", cards.len());
            }
            let command = app.apple.as_mut().unwrap().read(
                Read::Albums,
                "/v1/me/library/albums?limit=100".into(),
                0,
            );
            host.send(command.to_string());
            let response = await_event("response", command["id"].as_u64());
            assert!(response["error"].is_null(), "Library album read failed");
            let rows = response["data"]["data"].as_array().unwrap();
            let dated = rows
                .iter()
                .filter(|row| models::added_at(row).is_some())
                .count();
            let total = rows.len();
            app.apple_response(&response);
            assert_eq!(app.library.albums.items.len(), total);
            assert_eq!(
                app.library
                    .albums
                    .items
                    .iter()
                    .filter(|row| row.added_at.is_some())
                    .count(),
                dated
            );
            eprintln!("Library albums: {total} loaded rows, {dated} parsed add dates");
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(check));
        host.shutdown();
        app.backend.shutdown();
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }

    #[test]
    fn apple_home_reads_paginate_refresh_and_ignore_stale_answers() {
        use crate::apple::HomeShelf;
        let mut app = super::super::tests::test_app("apple-home");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        app.apple = Some(state);
        app.library.playlists = Loadable::Loaded(Vec::new());
        app.ensure_loaded(Page::Home);
        assert_eq!(app.apple.as_ref().unwrap().reads.len(), 4);
        app.ensure_loaded(Page::Home);
        assert_eq!(app.apple.as_ref().unwrap().reads.len(), 4);
        let id = |app: &App, shelf| {
            *app.apple
                .as_ref()
                .unwrap()
                .reads
                .iter()
                .find(|(_, (read, _))| *read == Read::Home(shelf))
                .unwrap()
                .0
        };
        let recent_id = id(&app, HomeShelf::Recent);
        let upload = json!({"id":"i.upload","type":"library-songs","attributes":{"name":"Upload"}});
        let song = json!({"id":"123","type":"songs","attributes":{"name":"Catalog", "playParams":{"id":"123","kind":"song"}}});
        app.apple_response(&json!({"id":recent_id,"data":{"data":[upload,song],"next":"/v1/me/recent/played?offset=10"}}));
        let state = app.apple.as_ref().unwrap();
        let cards = state.home[&HomeShelf::Recent].get().unwrap();
        assert_eq!(cards.len(), 2);
        assert!(!cards[0].playable);
        assert_eq!(cards[0].uri, "apple:track:library.i.upload");
        assert_eq!(state.known_songs[&cards[0].uri].item.id, "i.upload");
        app.load_more(Page::Home);
        let next = id(&app, HomeShelf::Recent);
        app.apple_response(&json!({"id":next,"error":"offline"}));
        assert_eq!(
            app.apple.as_ref().unwrap().home[&HomeShelf::Recent]
                .get()
                .unwrap()
                .len(),
            2
        );
        app.load_more(Page::Home);
        let next = id(&app, HomeShelf::Recent);
        app.apple_response(
            &json!({"id":next,"data":{"data":[song],"next":"/v1/me/recent/played?offset=10"}}),
        );
        assert!(
            !app.apple
                .as_ref()
                .unwrap()
                .next_reads
                .contains_key(&Read::Home(HomeShelf::Recent)),
            "repeated pagination stops"
        );
        let recommendation = id(&app, HomeShelf::Recommendations);
        app.apple_response(&json!({"id":recommendation,"data":{"data":[{"id":"rec","relationships":{"contents":{"data":[song,{"id":"station","type":"stations"}]}}}]}}));
        assert_eq!(
            app.apple.as_ref().unwrap().home[&HomeShelf::Recommendations]
                .get()
                .unwrap()
                .len(),
            1
        );
        let added = id(&app, HomeShelf::Added);
        app.apple_response(&json!({"id":added,"data":{"data":[]}}));
        assert!(
            app.apple.as_ref().unwrap().home[&HomeShelf::Added]
                .get()
                .unwrap()
                .is_empty()
        );
        let stale = id(&app, HomeShelf::HeavyRotation);
        app.reload(Page::Home);
        assert_eq!(
            app.apple.as_ref().unwrap().home[&HomeShelf::Recent]
                .get()
                .unwrap()
                .len(),
            2,
            "refresh keeps shown cards"
        );
        app.apple_response(&json!({"id":stale,"data":{"data":[song]}}));
        assert!(
            app.apple.as_ref().unwrap().home[&HomeShelf::HeavyRotation]
                .get()
                .is_none()
        );
        let failed = id(&app, HomeShelf::HeavyRotation);
        app.apple_response(&json!({"id":failed,"error":"offline"}));
        assert!(matches!(
            app.apple.as_ref().unwrap().home[&HomeShelf::HeavyRotation],
            Loadable::Failed(_)
        ));
        let late = id(&app, HomeShelf::Recent);
        app.apple.as_mut().unwrap().clear_account();
        app.apple_response(&json!({"id":late,"data":{"data":[song]}}));
        assert!(app.apple.as_ref().unwrap().home.is_empty());
        app.backend.shutdown();
    }
    #[test]
    fn album_playlist_add_waits_for_every_page_preserves_repeats_and_cancels_on_signout() {
        let mut app = super::super::tests::test_app("apple-album-playlist");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        state.storefront = "us".into();
        app.apple = Some(state);
        app.library.playlists = Loadable::Loaded(Vec::new());
        let uri = "apple:album:library.l.test";
        let resource =
            json!({"id":"i.upload","type":"library-songs","attributes":{"name":"Upload"}});
        let ctx = egui::Context::default();
        app.apply(
            Action::AddAlbumToPlaylist {
                uri: "spotify:album:foreign".into(),
                label: "Foreign".into(),
                playlist: None,
            },
            &ctx,
        );
        assert!(!app.playlist_busy);
        assert!(app.apple.as_ref().unwrap().pending_album_playlist.is_none());
        app.apply(
            Action::AddAlbumToPlaylist {
                uri: uri.into(),
                label: "Album".into(),
                playlist: None,
            },
            &ctx,
        );
        assert!(app.playlist_busy);
        let tracks_read = |app: &App| {
            *app.apple
                .as_ref()
                .unwrap()
                .reads
                .iter()
                .find(|(_, (target, _))| *target == Read::AlbumTracks("library.l.test".into()))
                .unwrap()
                .0
        };
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[resource.clone()],"next":"/v1/me/library/albums/l.test/tracks?offset=1"}}));
        app.apple_finish_album_playlist();
        assert!(app.dialog.is_none());
        assert!(app.apple.as_ref().unwrap().playlist_creates.is_empty());
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[resource.clone()]}}));
        app.apple_finish_album_playlist();
        let Some(Dialog::CreatePlaylist { name, add_uris, .. }) = &app.dialog else {
            panic!("Complete album should open a playlist draft")
        };
        assert_eq!(name, "Album");
        assert_eq!(
            add_uris,
            &vec!["apple:track:library.i.upload".to_owned(); 2]
        );
        assert!(!app.playlist_busy);
        app.dialog = None;
        app.apple
            .as_mut()
            .unwrap()
            .writable_playlists
            .insert("library.p.test".into());
        app.playlist_pages
            .entry("library.p.test".into())
            .or_default()
            .items
            .absorb(0, page(Vec::new(), &json!({}), 0));
        app.apply(
            Action::AddAlbumToPlaylist {
                uri: uri.into(),
                label: "Album".into(),
                playlist: Some(("library.p.test".into(), "Playlist".into())),
            },
            &ctx,
        );
        assert_eq!(app.playlist_pages["library.p.test"].items.items.len(), 2);
        assert!(app.playlist_busy);
        app.apply(Action::SignOut, &ctx);
        assert!(app.apple.as_ref().unwrap().pending_album_playlist.is_none());
        assert!(app.apple.as_ref().unwrap().playlist_appends.is_empty());
        app.apple.as_mut().unwrap().authorized = true;
        app.apple.as_mut().unwrap().ready = true;
        app.library.playlists = Loadable::Loaded(Vec::new());
        app.apply(
            Action::AddAlbumToPlaylist {
                uri: uri.into(),
                label: "Album".into(),
                playlist: None,
            },
            &ctx,
        );
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[resource.clone()],"next":"https://example.com/invalid"}}));
        app.apple_finish_album_playlist();
        assert!(!app.playlist_busy);
        assert!(app.dialog.is_none());
        app.album_pages.remove("library.l.test");
        app.apply(
            Action::AddAlbumToPlaylist {
                uri: uri.into(),
                label: "Album".into(),
                playlist: None,
            },
            &ctx,
        );
        let stale = tracks_read(&app);
        app.apply(Action::SignOut, &ctx);
        app.apple_response(&json!({"id":stale,"data":{"data":[resource]}}));
        app.apple_finish_album_playlist();
        assert!(app.dialog.is_none());
        assert!(!app.playlist_busy);
        app.backend.shutdown();
    }
    #[test]
    fn playlist_append_is_permission_scoped_atomic_and_preserves_duplicate_uploads() {
        let mut app = super::super::tests::test_app("apple-playlist-append");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        let catalog_row = json!({"id":"123","type":"songs","attributes":{"name":"Catalog","playParams":{"id":"456","kind":"song"}}});
        let upload_row =
            json!({"id":"i.upload","type":"library-songs","attributes":{"name":"Upload"}});
        state.songs = vec![
            models::song(&catalog_row).unwrap(),
            models::song(&upload_row).unwrap(),
        ];
        let upload = PlayableItem::Track(state.songs[1].track());
        let id = "library.p.test";
        app.apple = Some(state);
        let read = app.apple.as_mut().unwrap().read(
            Read::Playlists,
            "/v1/me/library/playlists?limit=100".into(),
            0,
        );
        app.apple_response(&json!({"id":read["id"],"data":{"data":[
            {"id":"p.test","type":"library-playlists","attributes":{"name":"Writable","canEdit":true}},
            {"id":"p.locked","type":"library-playlists","attributes":{"name":"Locked","canEdit":false}},
            {"id":"p.unknown","type":"library-playlists","attributes":{"name":"Unknown"}},
            {"id":"pl.catalog","type":"playlists","attributes":{"name":"Catalog","canEdit":true}}
        ]}}));
        assert_eq!(
            app.editable_playlists(),
            vec![(id.into(), "Writable".into())]
        );
        let playlist = app.library.playlists.get().unwrap()[0].clone();
        assert!(app.can_append_playlist(&playlist));
        assert!(!app.can_edit_playlist(&playlist));
        app.playlist_pages.entry(id.into()).or_default().playlist = Loadable::Loaded(playlist);
        let read = app.apple.as_mut().unwrap().read(
            Read::PlaylistTracks(id.into()),
            "/v1/me/library/playlists/p.test/tracks?limit=100".into(),
            0,
        );
        app.apple_response(&json!({"id":read["id"],"data":{"data":[catalog_row.clone()]}}));
        let before = app.playlist_pages[id].items.items.clone();
        app.apple_add_to_playlist(
            "library.p.locked",
            "Locked",
            std::slice::from_ref(&upload),
            true,
        );
        assert!(app.apple.as_ref().unwrap().playlist_appends.is_empty());
        let unknown = PlayableItem::Track(Track {
            uri: "apple:song:library.i.unknown".into(),
            ..Default::default()
        });
        app.apple_add_to_playlist(id, "Writable", &[upload.clone(), unknown], true);
        assert_eq!(app.playlist_pages[id].items.items, before);
        let stale = app.apple.as_mut().unwrap().read(
            Read::PlaylistTracks(id.into()),
            "/v1/me/library/playlists/p.test/tracks?limit=100".into(),
            0,
        );
        let items = vec![upload.clone(), upload.clone()];
        app.apple_add_to_playlist(id, "Writable", &items, true);
        assert_eq!(app.playlist_pages[id].items.items.len(), 3);
        assert_eq!(
            app.playlist_pages[id].playlist.get().unwrap().track_total(),
            3
        );
        assert!(app.playlist_busy);
        app.apple_response(&json!({"id":stale["id"],"data":{"data":[]}}));
        assert_eq!(app.playlist_pages[id].items.items.len(), 3);
        let request = *app
            .apple
            .as_ref()
            .unwrap()
            .playlist_appends
            .keys()
            .next()
            .unwrap();
        app.apple_response(&json!({"id":request,"error":"Example rejection"}));
        assert!(!app.playlist_busy);
        assert_eq!(app.playlist_pages[id].items.items, before);
        assert_eq!(app.library.playlists.get().unwrap()[0].track_total(), 1);
        assert!(app.apple.as_ref().unwrap().playlist_confirms.is_empty());
        app.apple_add_to_playlist(id, "Writable", &items, true);
        let request = *app
            .apple
            .as_ref()
            .unwrap()
            .playlist_appends
            .keys()
            .next()
            .unwrap();
        app.apple_response(&json!({"id":request,"data":null}));
        assert!(!app.playlist_busy);
        let ctx = egui::Context::default();
        app.apple_recheck_playlists(&ctx, Instant::now() + Duration::from_secs(4));
        let tracks_read = |app: &App| {
            *app.apple
                .as_ref()
                .unwrap()
                .reads
                .iter()
                .find(|(_, (target, _))| *target == Read::PlaylistTracks(id.into()))
                .unwrap()
                .0
        };
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[catalog_row.clone(), upload_row.clone()]}}));
        assert_eq!(
            app.playlist_pages[id].items.items.len(),
            3,
            "one uploaded occurrence must not confirm two"
        );
        assert!(
            app.apple
                .as_ref()
                .unwrap()
                .playlist_confirms
                .contains_key(id)
        );
        app.apple_recheck_playlists(&ctx, Instant::now() + Duration::from_secs(4));
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[catalog_row, upload_row.clone(), upload_row]}}));
        assert!(
            !app.apple
                .as_ref()
                .unwrap()
                .playlist_confirms
                .contains_key(id)
        );
        app.apple_add_to_playlist(id, "Writable", &items, true);
        assert!(
            matches!(&app.dialog, Some(Dialog::ConfirmPlaylistDuplicates { duplicate_uris, .. }) if duplicate_uris.len() == 2)
        );
        assert!(app.apple.as_ref().unwrap().playlist_appends.is_empty());
        app.apply(Action::CloseDialog, &ctx);
        assert!(app.dialog.is_none());
        app.apply(
            Action::ConfirmAddToPlaylist {
                playlist_id: id.into(),
                playlist_name: "Writable".into(),
                items,
                position: None,
            },
            &ctx,
        );
        assert_eq!(app.playlist_pages[id].items.items.len(), 5);
        let request = *app
            .apple
            .as_ref()
            .unwrap()
            .playlist_appends
            .keys()
            .next()
            .unwrap();
        app.apply(Action::SignOut, &ctx);
        app.apple_response(&json!({"id":request,"data":null}));
        assert!(!app.playlist_busy);
        assert!(app.playlist_pages.is_empty());
        assert!(app.apple.as_ref().unwrap().writable_playlists.is_empty());
        app.backend.shutdown();
    }
    #[test]
    fn playlist_append_waits_for_all_destination_pages_and_stops_on_bad_continuation() {
        let mut app = super::super::tests::test_app("apple-playlist-append-pages");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        let song_row = json!({"id":"123","type":"songs","attributes":{"name":"Song","playParams":{"id":"123","kind":"song"}}});
        state.songs = vec![models::song(&song_row).unwrap()];
        let items = vec![PlayableItem::Track(state.songs[0].track())];
        let id = "library.p.test";
        state.writable_playlists.insert(id.into());
        app.apple = Some(state);
        app.library.playlists = Loadable::Loaded(Vec::new());
        app.apple_add_to_playlist(id, "Test", &items, true);
        assert!(app.playlist_busy);
        let tracks_read = |app: &App| {
            *app.apple
                .as_ref()
                .unwrap()
                .reads
                .iter()
                .find(|(_, (target, _))| *target == Read::PlaylistTracks(id.into()))
                .unwrap()
                .0
        };
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[song_row.clone()],"next":"/v1/me/library/playlists/p.test/tracks?offset=1"}}));
        app.apple_finish_playlist_add();
        assert!(app.apple.as_ref().unwrap().playlist_appends.is_empty());
        assert!(app.apple.as_ref().unwrap().pending_playlist_add.is_some());
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[],"next":"/v1/me/library/playlists/p.test/tracks?offset=1"}}));
        app.apple_finish_playlist_add();
        assert!(!app.playlist_busy);
        assert!(app.apple.as_ref().unwrap().pending_playlist_add.is_none());
        assert!(app.apple.as_ref().unwrap().playlist_appends.is_empty());
        assert!(app.playlist_pages[id].items.error.is_some());
        app.playlist_pages.remove(id);
        app.apple_add_to_playlist(id, "Test", &items, true);
        let pending_read = tracks_read(&app);
        app.apply(Action::SignOut, &egui::Context::default());
        app.apple_response(&json!({"id":pending_read,"data":{"data":[song_row]}}));
        app.apple_finish_playlist_add();
        assert!(!app.playlist_busy);
        assert!(app.apple.as_ref().unwrap().pending_playlist_add.is_none());
        assert!(app.playlist_pages.is_empty());
        app.backend.shutdown();
    }
    #[test]
    fn playlist_creation_keeps_occurrences_until_all_apple_pages_confirm_them() {
        let mut app = super::super::tests::test_app("apple-playlist-create");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        state.loading = false;
        let catalog = models::song(&json!({"id":"123","type":"songs","attributes":{"name":"Catalog","playParams":{"id":"456","kind":"song"}}})).unwrap();
        let upload = models::song(
            &json!({"id":"i.upload","type":"library-songs","attributes":{"name":"Upload"}}),
        )
        .unwrap();
        let uris = vec![catalog.uri(), upload.uri(), upload.uri()];
        state.queue = vec![catalog, upload.clone(), upload];
        state.index = Some(0);
        state.order.upcoming = vec![1, 2];
        let read = state.read(Read::Albums, "/v1/me/library/albums?limit=100".into(), 0);
        let (_, payload) = state.create_playlist("Test", false, &uris).unwrap();
        assert!(payload["id"].as_u64().unwrap() > read["id"].as_u64().unwrap());
        assert_eq!(payload["items"][0]["id"], "123");
        assert_eq!(payload["items"][0]["playParams"]["id"], "456");
        assert_eq!(payload["items"][1], payload["items"][2]);
        state.playlist_creates.clear();
        app.apple = Some(state);
        assert_eq!(app.queue_playlist_uris(), uris);
        let ctx = egui::Context::default();
        app.apply(
            Action::CreatePlaylist {
                name: "Test".into(),
                public: false,
                add_uris: uris.clone(),
            },
            &ctx,
        );
        let (&request, temporary) = app
            .apple
            .as_ref()
            .unwrap()
            .playlist_creates
            .iter()
            .next()
            .unwrap();
        let temporary = temporary.clone();
        assert_eq!(app.library.playlists.get().unwrap()[0].id, temporary);
        assert_eq!(
            app.library.playlists.get().unwrap()[0].owner_name(),
            "Apple Music"
        );
        assert_eq!(app.playlist_pages[&temporary].items.items.len(), 3);
        app.reload(Page::Playlist(temporary.clone()));
        assert!(app.playlist_pages.contains_key(&temporary));
        let read = app.apple.as_mut().unwrap().read(
            Read::Playlists,
            "/v1/me/library/playlists?limit=100".into(),
            0,
        );
        app.apple_response(&json!({"id":read["id"],"data":{"data":[]}}));
        assert_eq!(app.library.playlists.get().unwrap()[0].id, temporary);
        app.assumed_context = Some(AssumedContext {
            uri: format!("apple:playlist:{temporary}"),
            shuffle: None,
            at: Instant::now(),
        });
        app.apple_response(&json!({"id":request,"data":{"data":[{"id":"p.test","type":"library-playlists","attributes":{"name":"Test","isPublic":false,"canEdit":true}}]}}));
        let id = "library.p.test";
        assert!(!app.playlist_busy);
        assert!(app.dialog.is_none());
        assert_eq!(*app.page(), Page::Playlist(id.into()));
        assert_eq!(
            app.playlist_pages[id].playlist.get().unwrap().owner_name(),
            "Apple Music"
        );
        assert_eq!(
            app.assumed_context.as_ref().unwrap().uri,
            format!("apple:playlist:{id}")
        );
        assert!(
            app.apple
                .as_ref()
                .unwrap()
                .playlist_confirms
                .contains_key(id)
        );
        // An incomplete/stale response never replaces the submitted queue.
        app.reload(Page::Playlist(id.into()));
        app.apple_recheck_playlists(&ctx, Instant::now() + Duration::from_secs(5));
        let tracks_read = |app: &App| {
            *app.apple
                .as_ref()
                .unwrap()
                .reads
                .iter()
                .find(|(_, (target, _))| *target == Read::PlaylistTracks(id.into()))
                .unwrap()
                .0
        };
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[],"next":"/v1/me/library/playlists/p.test/tracks?offset=0"}}));
        assert_eq!(app.playlist_pages[id].items.items.len(), 3);
        assert!(
            !app.apple
                .as_ref()
                .unwrap()
                .reads
                .values()
                .any(|(target, _)| *target == Read::PlaylistTracks(id.into())),
            "an empty continuation must not spin indefinitely"
        );
        assert!(
            !app.apple
                .as_ref()
                .unwrap()
                .next_reads
                .contains_key(&Read::PlaylistTracks(id.into()))
        );
        app.apple_recheck_playlists(&ctx, Instant::now() + Duration::from_secs(5));
        let matched = json!({"id":"i.matched","type":"library-songs","attributes":{"name":"Catalog","playParams":{"id":"i.matched","isLibrary":true,"catalogId":"123"}}});
        let wrong = json!({"id":"i.other","type":"library-songs","attributes":{"name":"Upload","playParams":{"id":"i.other","isLibrary":true,"catalogId":"123"}}});
        app.apple_response(
            &json!({"id":tracks_read(&app),"data":{"data":[matched.clone(),wrong.clone(),wrong]}}),
        );
        assert!(
            app.apple
                .as_ref()
                .unwrap()
                .playlist_confirms
                .contains_key(id),
            "an upload cannot be replaced by a catalog match"
        );
        app.apple_recheck_playlists(&ctx, Instant::now() + Duration::from_secs(5));
        app.apple_response(&json!({"id":tracks_read(&app),"error":"Not ready"}));
        assert!(app.playlist_pages[id].items.error.is_some());
        assert!(
            app.apple.as_ref().unwrap().playlist_recheck_at.is_none(),
            "automatic confirmation stops after three attempts"
        );
        app.apply(Action::RetryWindow(Page::Playlist(id.into())), &ctx);
        app.apple_recheck_playlists(&ctx, Instant::now() + Duration::from_secs(5));
        app.apple_response(&json!({"id":tracks_read(&app),"data":{"data":[matched],"next":"/v1/me/library/playlists/p.test/tracks?offset=1"}}));
        assert_eq!(app.playlist_pages[id].items.items.len(), 3);
        let upload = json!({"id":"i.upload","type":"library-songs","attributes":{"name":"Upload"}});
        app.apple_response(
            &json!({"id":tracks_read(&app),"data":{"data":[upload.clone(),upload]}}),
        );
        assert!(
            !app.apple
                .as_ref()
                .unwrap()
                .playlist_confirms
                .contains_key(id)
        );
        assert_eq!(app.playlist_pages[id].items.items.len(), 3);
        assert!(app.playlist_pages[id].items.is_complete());
        // A slower library listing can still predate the successful track read.
        let listing = *app
            .apple
            .as_ref()
            .unwrap()
            .reads
            .iter()
            .find(|(_, (target, _))| *target == Read::Playlists)
            .unwrap()
            .0;
        app.apple_response(&json!({"id":listing,"data":{"data":[]}}));
        assert_eq!(app.library.playlists.get().unwrap()[0].id, id);
        assert_eq!(
            app.queue_playlist_uris(),
            uris,
            "playlist writes do not alter playback"
        );
        app.backend.shutdown();
    }

    #[test]
    fn playlist_creation_failure_and_signout_cannot_leave_a_row_or_busy_state() {
        let mut app = super::super::tests::test_app("apple-playlist-failure");
        let mut state = crate::apple::State::default();
        state.authorized = true;
        state.ready = true;
        state.loading = false;
        app.apple = Some(state);
        app.apple_create_playlist("", false, &[]);
        app.apple_create_playlist("Missing", false, &["apple:track:library.i.missing".into()]);
        assert!(app.apple.as_ref().unwrap().playlist_creates.is_empty());
        assert!(!app.playlist_busy);
        app.apple_create_playlist("Retry", false, &[]);
        let (&request, temporary) = app
            .apple
            .as_ref()
            .unwrap()
            .playlist_creates
            .iter()
            .next()
            .unwrap();
        let temporary = temporary.clone();
        app.open(Page::Playlist(temporary.clone()));
        app.apple_response(&json!({"id":request,"error":"Network error"}));
        assert!(!app.playlist_busy);
        assert!(matches!(app.dialog, Some(Dialog::CreatePlaylist { .. })));
        assert!(!app.playlist_pages.contains_key(&temporary));
        assert!(app.library.playlists.get().unwrap().is_empty());
        assert_eq!(*app.page(), Page::Home);
        app.apple_create_playlist("Later", true, &[]);
        let request = *app
            .apple
            .as_ref()
            .unwrap()
            .playlist_creates
            .keys()
            .next()
            .unwrap();
        app.apply(Action::SignOut, &egui::Context::default());
        app.apple_response(&json!({"id":request,"data":{"data":[{"id":"p.late","type":"library-playlists","attributes":{"name":"Later"}}]}}));
        assert!(!app.playlist_busy);
        assert!(app.playlist_pages.is_empty());
        assert!(app.dialog.is_none());
        assert!(app.apple.as_ref().unwrap().playlist_creates.is_empty());
        assert!(app.apple.as_ref().unwrap().playlist_confirms.is_empty());
        app.backend.shutdown();
    }
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
