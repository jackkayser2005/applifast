//! Optimistic additions with exact confirmation and ordered metadata reads.
use super::{Song, State, cache};
use applifast_playback_probe::protocol::Command;
use serde_json::Value;

pub(super) struct Update {
    after: u64,
    value: Option<bool>,
    pending: bool,
}

impl State {
    pub fn song_favorite(&self, song: &Song) -> Option<bool> {
        self.favorite_updates
            .get(&song.uri())
            .map_or(song.in_favorites, |update| update.value)
    }

    pub fn favorite_status(&self, uri: &str) -> Option<bool> {
        if let Some(update) = self.favorite_updates.get(uri) {
            return update.value;
        }
        self.songs
            .iter()
            .find(|song| song.uri() == uri)
            .or_else(|| self.find_song(uri))
            .and_then(|song| song.in_favorites)
    }

    pub fn favorite_songs(&mut self, uris: &[String]) -> Result<Option<Value>, String> {
        if !self.ready || !self.authorized {
            return Err("Sign in to Apple Music before adding favorites.".into());
        }
        let songs = self
            .songs_for_uris(uris)
            .ok_or("Reload the selected songs before adding favorites.")?;
        let mut seen = std::collections::HashSet::new();
        let songs = songs
            .into_iter()
            .filter(|song| seen.insert(song.uri()))
            .filter(|song| self.favorite_status(&song.uri()) != Some(true))
            .collect::<Vec<_>>();
        if songs.is_empty() {
            return Ok(None);
        }
        let id = self.read_serial + 1;
        let command = Command::FavoriteSongs {
            id,
            items: songs.iter().map(|song| song.item.clone()).collect(),
        };
        command.validate()?;
        let changes = songs
            .iter()
            .map(|song| (song.uri(), self.favorite_status(&song.uri())))
            .collect::<Vec<_>>();
        self.read_serial = id;
        for (uri, _) in &changes {
            self.favorite_updates.insert(
                uri.clone(),
                Update {
                    after: id,
                    value: Some(true),
                    pending: true,
                },
            );
        }
        self.favorite_writes.insert(id, changes);
        Ok(Some(
            serde_json::to_value(command).expect("favorite command serializes"),
        ))
    }

    /// None means this response does not belong to a pending favorites request.
    pub fn complete_favorites(&mut self, event: &Value) -> Option<bool> {
        if event["type"] != "response" || event["session"].as_u64() != Some(self.session) {
            return None;
        }
        let changes = self.favorite_writes.remove(&event["id"].as_u64()?)?;
        let rows = event["data"]["data"].as_array();
        let mut all_confirmed = true;
        for (uri, previous) in changes {
            let resource = uri
                .strip_prefix("apple:track:")
                .expect("validated song URI");
            let (kind, id) = resource.split_once('.').expect("typed song URI");
            let resource_type = if kind == "library" {
                "library-songs"
            } else {
                "songs"
            };
            let matches = rows
                .into_iter()
                .flatten()
                .filter(|row| row["id"] == id && row["type"] == resource_type)
                .collect::<Vec<_>>();
            let confirmed = matches.len() == 1 && matches[0]["attributes"]["inFavorites"] == true;
            all_confirmed &= confirmed;
            self.favorite_updates.insert(
                uri,
                Update {
                    after: self.read_serial,
                    value: if confirmed { Some(true) } else { previous },
                    pending: false,
                },
            );
        }
        self.cache_dirty = true;
        Some(all_confirmed)
    }

    pub fn observe_favorites(&mut self, read_id: u64, songs: &mut [Song]) {
        for song in songs {
            if let Some(update) = self.favorite_updates.get_mut(&song.uri()) {
                if !update.pending && read_id > update.after {
                    update.value = song.in_favorites;
                    update.after = read_id;
                }
                song.in_favorites = update.value;
            }
        }
    }

    pub(super) fn cache_favorites(&self, snapshot: &mut cache::Snapshot) {
        let pending = self
            .favorite_writes
            .values()
            .flatten()
            .map(|(uri, previous)| (uri.as_str(), *previous))
            .collect::<std::collections::HashMap<_, _>>();
        for song in snapshot.songs.iter_mut().chain(&mut snapshot.queue) {
            let uri = song.uri();
            if let Some(previous) = pending.get(uri.as_str()) {
                song.in_favorites = *previous;
            } else if let Some(update) = self.favorite_updates.get(&uri) {
                song.in_favorites = update.value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn state() -> State {
        State { ready: true, authorized: true, cache_checked: true,
            account_tag: Some("a".repeat(64)), storefront: "us".into(),
            songs: [json!({"id":"123","type":"songs","attributes":{"name":"Catalog","inFavorites":false}}),
                json!({"id":"i.upload","type":"library-songs","attributes":{"name":"Upload","playParams":{"id":"i.upload","isLibrary":true,"kind":"song","catalogId":"123"}}})]
                .iter().map(|row| super::super::models::song(row).unwrap()).collect(), ..State::default() }
    }

    #[test]
    fn additions_preserve_typed_ids_deduplicate_and_do_not_persist_pending_hearts() {
        let mut state = state();
        let uris = state.songs.iter().map(Song::uri).collect::<Vec<_>>();
        let command = state
            .favorite_songs(&[uris[0].clone(), uris[1].clone(), uris[1].clone()])
            .unwrap()
            .unwrap();
        assert_eq!(command["items"].as_array().unwrap().len(), 2);
        assert_eq!(command["items"][1]["id"], "i.upload");
        assert_eq!(command["items"][1]["playParams"]["catalogId"], "123");
        for uri in &uris {
            assert_eq!(state.favorite_status(uri), Some(true));
        }
        assert!(
            state.favorite_songs(&uris).unwrap().is_none(),
            "Repeated clicks must not repeat the write"
        );
        let mut stale = state.songs.clone();
        state.observe_favorites(0, &mut stale);
        state.songs = stale;
        let cache = state.cache_snapshot(true).unwrap();
        assert_eq!(cache.songs[0].in_favorites, Some(false));
        assert_eq!(cache.songs[1].in_favorites, None);
        assert!(
            state
                .favorite_songs(&["apple:track:catalog.999".into()])
                .is_err()
        );
    }

    #[test]
    fn partial_confirmation_rolls_back_exact_members_and_new_reads_win_in_order() {
        let mut state = state();
        let uris = state.songs.iter().map(Song::uri).collect::<Vec<_>>();
        let command = state.favorite_songs(&uris).unwrap().unwrap();
        let old_read = state.library_request(None)["id"].as_u64().unwrap();
        let response = json!({"type":"response","session":1,"id":command["id"],"error":"partial",
            "data":{"data":[{"id":"123","type":"songs","attributes":{"inFavorites":true}},
                {"id":"i.upload","type":"songs","attributes":{"inFavorites":true}}]}});
        assert_eq!(state.complete_favorites(&response), Some(false));
        assert_eq!(state.favorite_status(&uris[0]), Some(true));
        assert_eq!(state.favorite_status(&uris[1]), None);
        assert!(state.complete_favorites(&response).is_none());
        let mut stale = state.songs.clone();
        state.observe_favorites(old_read, &mut stale);
        assert_eq!(stale[0].in_favorites, Some(true));
        let cache = state.cache_snapshot(true).unwrap();
        assert_eq!(cache.songs[0].in_favorites, Some(true));
        assert_eq!(cache.songs[1].in_favorites, None);
        let new_read = state.library_request(None)["id"].as_u64().unwrap();
        let mut fresh = cache.songs.clone();
        fresh[0].in_favorites = Some(false);
        state.observe_favorites(new_read, &mut fresh);
        assert_eq!(
            state.favorite_status(&uris[0]),
            Some(false),
            "A newer server read can reflect an external change"
        );
        state.observe_favorites(old_read, &mut stale);
        assert_eq!(state.favorite_status(&uris[0]), Some(false));
        let retry = state.favorite_songs(&uris).unwrap().unwrap();
        state.session += 1;
        state.clear_account();
        assert!(
            state
                .complete_favorites(&json!({"type":"response","session":1,"id":retry["id"]}))
                .is_none()
        );
        assert!(state.favorite_updates.is_empty());
        assert!(state.favorite_writes.is_empty());
    }

    #[test]
    fn failed_write_restores_known_and_unknown_state_without_accepting_ambiguous_resources() {
        for rows in [
            json!([]),
            json!([{"id":"123","type":"songs","attributes":{"inFavorites":"true"}}]),
            json!([{"id":"123","type":"songs","attributes":{"inFavorites":true}},
                {"id":"123","type":"songs","attributes":{"inFavorites":true}}]),
        ] {
            let mut state = state();
            let uris = state.songs.iter().map(Song::uri).collect::<Vec<_>>();
            let command = state.favorite_songs(&uris).unwrap().unwrap();
            assert_eq!(
                state.complete_favorites(
                    &json!({"type":"response","session":1,"id":command["id"],"data":{"data":rows}})
                ),
                Some(false)
            );
            assert_eq!(state.favorite_status(&uris[0]), Some(false));
            assert_eq!(state.favorite_status(&uris[1]), None);
        }
    }

    #[test]
    fn unauthorized_or_oversized_selection_cannot_partially_change_hearts() {
        let mut state = state();
        let mut song = state.songs[0].clone();
        state.songs = (1..=101)
            .map(|id| {
                song.item.id = id.to_string();
                song.clone()
            })
            .collect();
        let uris = state.songs.iter().map(Song::uri).collect::<Vec<_>>();
        assert!(state.favorite_songs(&uris).is_err());
        assert!(state.favorite_updates.is_empty());
        assert!(state.favorite_writes.is_empty());
        state.authorized = false;
        assert!(state.favorite_songs(&uris[..1]).is_err());
        assert_eq!(state.favorite_status(&uris[0]), Some(false));
    }
}
