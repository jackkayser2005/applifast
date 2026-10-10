//! The temporary playback probe's input boundary. No Spotify IDs or token logging.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Catalog,
    Library,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaybackItem {
    pub kind: ItemKind,
    pub id: String,
    pub play_params: Option<Value>,
}

impl PlaybackItem {
    pub fn validate(&self) -> Result<(), String> {
        let valid_id = !self.id.is_empty()
            && self.id.len() <= 128
            && match self.kind {
                ItemKind::Catalog => self.id.bytes().all(|byte| byte.is_ascii_digit()),
                ItemKind::Library => {
                    self.id.starts_with("i.")
                        && self.id.len() > 2
                        && self.id.bytes().all(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
                        })
                }
            };
        if !valid_id {
            return Err("Expected a catalog song ID or an i.* library song ID.".into());
        }
        if let Some(params) = &self.play_params {
            let playback_id = params.get("id").and_then(Value::as_str);
            if !params.is_object()
                || playback_id.is_none_or(|id| {
                    id.is_empty()
                        || id.len() > 128
                        || match self.kind {
                            ItemKind::Library => id != self.id,
                            ItemKind::Catalog => !id.bytes().all(|byte| byte.is_ascii_digit()),
                        }
                })
            {
                return Err("Playback parameters must retain a library ID or a valid Apple catalog playback ID.".into());
            }
            if matches!(self.kind, ItemKind::Library)
                && params.get("isLibrary").and_then(Value::as_bool) != Some(true)
            {
                return Err("Library playback parameters must have isLibrary=true.".into());
            }
        }
        Ok(())
    }
}

/// Indices identify occurrences, so two copies of one song remain distinct.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QueueOrder {
    pub upcoming: Vec<usize>,
    pub manual_count: usize,
    pub context: Vec<usize>,
    /// A bounded local Previous history, including consumed manual occurrences.
    #[serde(default)]
    pub history: Vec<usize>,
}

impl QueueOrder {
    pub fn validate(&self, len: usize, current: Option<usize>) -> Result<(), String> {
        let unique = |indices: &[usize]| {
            indices.len() <= len
                && indices.iter().all(|index| *index < len)
                && indices
                    .iter()
                    .copied()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    == indices.len()
        };
        if !unique(&self.upcoming)
            || !unique(&self.context)
            || self.manual_count > self.upcoming.len()
            || self.history.len() > 64
            || self.history.iter().any(|index| *index >= len)
            || current.is_some_and(|index| index >= len || self.upcoming.contains(&index))
            || self.upcoming[..self.manual_count]
                .iter()
                .any(|index| self.context.contains(index))
            || self.upcoming[self.manual_count..]
                .iter()
                .any(|index| !self.context.contains(index))
        {
            return Err("Invalid queue occurrence order.".into());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    Intent {
        generation: u64,
        command: Box<Command>,
    },
    Authorize,
    Library {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        next: Option<String>,
    },
    Request {
        id: u64,
        path: String,
    },
    CreatePlaylist {
        id: u64,
        name: String,
        public: bool,
        items: Vec<PlaybackItem>,
    },
    AppendPlaylist {
        id: u64,
        playlist: String,
        items: Vec<PlaybackItem>,
    },
    Play {
        items: Vec<PlaybackItem>,
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        generation: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        order: Option<QueueOrder>,
    },
    Queue {
        items: Vec<PlaybackItem>,
        index: Option<usize>,
        order: QueueOrder,
    },
    Restore {
        items: Vec<PlaybackItem>,
        index: Option<usize>,
        order: QueueOrder,
        seconds: f64,
        shuffle: bool,
        repeat: u8,
    },
    Jump {
        position: usize,
    },
    Select {
        index: usize,
        order: QueueOrder,
        playing: bool,
    },
    Pause,
    Resume,
    Next,
    Previous,
    Seek {
        seconds: f64,
    },
    Volume {
        value: f64,
    },
    Shuffle {
        enabled: bool,
    },
    Repeat {
        mode: u8,
    },
    SignOut,
    Probe,
    Shutdown,
}

impl Command {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Intent {
                generation,
                command,
            } => {
                if *generation > 9_007_199_254_740_991
                    || matches!(
                        **command,
                        Self::Intent { .. }
                            | Self::Authorize
                            | Self::Library { .. }
                            | Self::Request { .. }
                            | Self::CreatePlaylist { .. }
                            | Self::AppendPlaylist { .. }
                            | Self::SignOut
                            | Self::Shutdown
                    )
                {
                    return Err("Invalid playback intent.".into());
                }
                command.validate()
            }
            Self::Library { id: Some(id), .. } if *id > 9_007_199_254_740_991 => {
                Err("Library request ID exceeds the JavaScript integer range.".into())
            }
            Self::Library {
                next: Some(next), ..
            } if !valid_read_path(next) || !next.starts_with("/v1/me/library/songs?") => {
                Err("Only a next-page path for the song library is accepted.".into())
            }
            Self::Request { id, path } if *id > 9_007_199_254_740_991 || !valid_read_path(path) => {
                Err("Only an Apple Music catalog or library read is accepted.".into())
            }
            Self::CreatePlaylist {
                id, name, items, ..
            } => {
                if name.trim().is_empty() || name.len() > 1024 {
                    return Err("Enter a playlist name of at most 1024 bytes.".into());
                }
                validate_playlist_items(*id, items, true)
            }
            Self::AppendPlaylist {
                id,
                playlist,
                items,
            } => {
                if !valid_library_playlist_id(playlist) {
                    return Err("Choose a library playlist.".into());
                }
                validate_playlist_items(*id, items, false)
            }
            Self::Play {
                items,
                index,
                generation,
                order,
            } => {
                if generation.is_some_and(|value| value > 9_007_199_254_740_991) {
                    return Err("Playback generation exceeds the JavaScript integer range.".into());
                }
                if items.is_empty() || items.len() > 1000 || *index >= items.len() {
                    return Err("Choose an existing row in a queue of 1 to 1000 songs.".into());
                }
                items.iter().try_for_each(PlaybackItem::validate)?;
                if let Some(order) = order {
                    order.validate(items.len(), Some(*index))?;
                }
                Ok(())
            }
            Self::Restore {
                seconds, repeat, ..
            } if !seconds.is_finite() || *seconds < 0.0 || *repeat > 2 => {
                Err("Invalid saved playback settings.".into())
            }
            Self::Queue {
                items,
                index,
                order,
            }
            | Self::Restore {
                items,
                index,
                order,
                ..
            } => {
                if items.len() > 1000 {
                    return Err("The queue supports at most 1000 occurrences.".into());
                }
                items.iter().try_for_each(PlaybackItem::validate)?;
                order.validate(items.len(), *index)
            }
            Self::Jump { position } if *position >= 1000 => Err("Invalid queue row.".into()),
            Self::Select { index, order, .. } => order.validate(1000, Some(*index)),
            Self::Seek { seconds } if !seconds.is_finite() || *seconds < 0.0 => {
                Err("Seek position must be a finite nonnegative number.".into())
            }
            Self::Volume { value } if !value.is_finite() || !(0.0..=1.0).contains(value) => {
                Err("Volume must be between zero and one.".into())
            }
            Self::Repeat { mode } if *mode > 2 => Err("Repeat must be 0, 1, or 2.".into()),
            _ => Ok(()),
        }
    }
}

pub fn valid_library_playlist_id(id: &str) -> bool {
    id.starts_with("p.")
        && id.len() > 2
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn validate_playlist_items(id: u64, items: &[PlaybackItem], empty: bool) -> Result<(), String> {
    if id > 9_007_199_254_740_991 || items.len() > 1000 || (!empty && items.is_empty()) {
        return Err("A playlist request supports at most 1000 song occurrences.".into());
    }
    items.iter().try_for_each(PlaybackItem::validate)
}

/// Restrict MusicKit reads to known collections, never external URLs or credentials.
pub fn valid_read_path(path: &str) -> bool {
    if path.len() > 2048 || !path.starts_with("/v1/") || path.contains(['\\', '#']) {
        return false;
    }
    let bare = path.split('?').next().unwrap_or_default();
    let parts: Vec<_> = bare.trim_start_matches('/').split('/').collect();
    if parts.iter().any(|part| {
        part.is_empty()
            || *part == "."
            || *part == ".."
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    }) {
        return false;
    }
    let resources = ["songs", "albums", "artists", "playlists", "search"];
    let valid = match parts.as_slice() {
        ["v1", "me", "recent", "played"]
        | ["v1", "me", "history", "heavy-rotation"]
        | ["v1", "me", "recommendations"]
        | ["v1", "me", "library", "recently-added"] => true,
        ["v1", "me", "library", resource, tail @ ..] => {
            resources.contains(resource)
                && tail.len() <= 2
                && tail.last().is_none_or(|item| {
                    tail.len() < 2 || ["tracks", "albums", "artists"].contains(item)
                })
        }
        ["v1", "catalog", storefront, resource, tail @ ..] => {
            storefront.len() == 2
                && storefront.bytes().all(|byte| byte.is_ascii_lowercase())
                && resources.contains(resource)
                && (tail.len() <= 2
                    && tail.last().is_none_or(|item| {
                        tail.len() < 2 || ["tracks", "albums", "artists"].contains(item)
                    })
                    || matches!(tail, [_, "view", "top-songs"]))
        }
        _ => false,
    };
    valid
        && path.split_once('?').is_none_or(|(_, query)| {
            url::form_urlencoded::parse(query.as_bytes()).all(|(key, value)| {
                ["limit", "offset", "term", "types", "include"].contains(&key.as_ref())
                    || (key == "extend"
                        && value == "inFavorites"
                        && matches!(
                            parts.as_slice(),
                            ["v1", "me", "library", "songs"] | ["v1", "me", "library", "songs", _]
                        ))
            })
        })
}

pub fn validate_developer_token(token: &str) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock is before the Unix epoch.".to_string())?
        .as_secs();
    validate_token_at(token, now)
}

/// Read a bounded local JWT. Errors never contain file contents or signing inputs.
pub fn read_developer_token_file(path: &std::path::Path) -> Result<String, String> {
    if path
        .extension()
        .is_some_and(|extension| extension.as_encoded_bytes().eq_ignore_ascii_case(b"p8"))
    {
        return Err("Expected a signed MusicKit developer JWT, not a .p8 signing key.".into());
    }
    let file = std::fs::File::open(path).map_err(|_| "Cannot open developer token file.")?;
    let mut bytes = Vec::new();
    file.take(32769)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read developer token file.")?;
    if bytes.len() > 32768 {
        return Err("Developer token file exceeds 32 KiB.".into());
    }
    let token = String::from_utf8(bytes).map_err(|_| "Developer token must be UTF-8.")?;
    validate_developer_token(token.trim())?;
    Ok(token.trim().to_owned())
}

/// Prefer a valid protected grant; otherwise use only the executable's companion file.
pub fn startup_developer_token(
    stored: Option<String>,
    executable: &std::path::Path,
) -> Result<(String, bool), String> {
    if let Some(token) = stored.filter(|token| validate_developer_token(token).is_ok()) {
        return Ok((token, false));
    }
    let path = executable.with_file_name("developer-token.txt");
    let token = read_developer_token_file(&path).map_err(|_| {
        "App developer token is missing, invalid or expired. Download a fresh Applifast build, or import a renewed developer token.".to_string()
    })?;
    Ok((token, true))
}

// This only checks format and expiry. Apple verifies the signature and access.
fn validate_token_at(token: &str, now: u64) -> Result<(), String> {
    let invalid = || "Expected a signed MusicKit developer JWT, not a .p8 signing key.".to_string();
    if token.len() > 8192 || token.contains(char::is_whitespace) {
        return Err(invalid());
    }
    let parts: Vec<_> = token.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return Err(invalid());
    }
    let decode = |part: &str| -> Result<Value, String> {
        let bytes = URL_SAFE_NO_PAD.decode(part).map_err(|_| invalid())?;
        serde_json::from_slice(&bytes).map_err(|_| invalid())
    };
    let header = decode(parts[0])?;
    let claims = decode(parts[1])?;
    let signature = URL_SAFE_NO_PAD.decode(parts[2]).map_err(|_| invalid())?;
    if header.get("alg").and_then(Value::as_str) != Some("ES256")
        || header
            .get("kid")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || claims
            .get("iss")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || signature.len() != 64
    {
        return Err(invalid());
    }
    let expires = claims
        .get("exp")
        .and_then(Value::as_u64)
        .ok_or_else(invalid)?;
    let issued = claims
        .get("iat")
        .and_then(Value::as_u64)
        .ok_or_else(invalid)?;
    if issued > now.saturating_add(300) || expires <= issued || expires - issued > 15_777_000 {
        return Err("Developer token dates are invalid. Check the signing machine's clock.".into());
    }
    if expires <= now {
        return Err("Developer token expired. Generate and import a fresh token.".into());
    }
    if let Some(origins) = claims.get("origin") {
        let expected = "https://applifast.invalid";
        let permits_probe = origins.as_str() == Some(expected)
            || origins
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)));
        if !permits_probe {
            return Err("Developer token's origin must permit https://applifast.invalid.".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_reject_external_paths_and_credential_parameters() {
        for path in [
            "https://example.com/v1/me/library/songs",
            "/v1/me/library/albums/../songs",
            "/v1/me/library/albums/%2e%2e/songs",
            "/v1/me/library/songs?token=secret",
            "/v1/me/library/songs#fragment",
            "/v1/catalog/us/artists/id/anything",
            "/v1/me/recommendations/anything",
            "/v1/me/history/heavy-rotation?token=secret",
            "/v1/me/library/recently-added/anything",
            "/v1/me/library/songs?extend=authorization",
            "/v1/me/library/songs?extend=inFavorites,authorization",
            "/v1/catalog/us/songs?extend=inFavorites",
            "/v1/me/library/playlists/p.1/tracks?extend=inFavorites",
        ] {
            assert!(!valid_read_path(path), "{path}");
        }
        for path in [
            "/v1/me/library/playlists?limit=100",
            "/v1/me/library/playlists/p.1/tracks?offset=100",
            "/v1/catalog/us/artists/123/view/top-songs?limit=20",
            "/v1/catalog/us/search?term=Some%20song&types=songs,albums",
            "/v1/me/recent/played?types=albums,playlists&limit=10",
            "/v1/me/history/heavy-rotation?offset=10",
            "/v1/me/recommendations?limit=10",
            "/v1/me/library/recently-added?offset=10",
            "/v1/me/library/songs?limit=100&extend=inFavorites",
            "/v1/me/library/songs/i.upload?extend=inFavorites",
        ] {
            assert!(valid_read_path(path), "{path}");
        }
    }

    #[test]
    fn uploaded_ids_and_duplicate_occurrences_are_retained() {
        let command: Command = serde_json::from_value(json!({
            "type": "play", "index": 1, "items": [
                {"kind": "library", "id": "i.upload", "playParams": null},
                {"kind": "catalog", "id": "123", "playParams": null},
                {"kind": "library", "id": "i.upload", "playParams": null}
            ]
        }))
        .unwrap();
        command.validate().unwrap();
        let encoded = serde_json::to_value(command).unwrap();
        assert_eq!(encoded["items"][0]["id"], encoded["items"][2]["id"]);
        let item = PlaybackItem {
            kind: ItemKind::Library,
            id: "i.upload".into(),
            play_params: Some(json!({"id":"123", "isLibrary":true})),
        };
        assert!(item.validate().is_err());
    }

    #[test]
    fn playlist_writes_keep_song_occurrences_and_reject_other_routes() {
        let create = json!({"type":"createPlaylist","id":7,"name":"Queue","public":false,"items":[
            {"kind":"library","id":"i.upload","playParams":null},
            {"kind":"catalog","id":"123","playParams":{"id":"456","kind":"song"}},
            {"kind":"library","id":"i.upload","playParams":null}
        ]});
        let command: Command = serde_json::from_value(create.clone()).unwrap();
        command.validate().unwrap();
        assert_eq!(serde_json::to_value(command).unwrap(), create);
        for playlist in ["p.editable", "p.Mixed_1-2"] {
            let command: Command = serde_json::from_value(json!({"type":"appendPlaylist","id":8,
                "playlist":playlist,"items":[{"kind":"library","id":"i.upload","playParams":null}]})).unwrap();
            command.validate().unwrap();
        }
        for playlist in [
            "",
            "p.",
            "pl.catalog",
            "https://evil.example",
            "p.x/../tracks",
            "p.x?token=secret",
        ] {
            assert!(!valid_library_playlist_id(playlist), "{playlist}");
        }
        for patch in [
            json!({"name":" "}),
            json!({"name":"a".repeat(1025)}),
            json!({"id":9_007_199_254_740_992u64}),
            json!({"items":vec![create["items"][0].clone();1001]}),
        ] {
            let mut input = create.clone();
            for (key, value) in patch.as_object().unwrap() {
                input[key] = value.clone();
            }
            assert!(
                serde_json::from_value::<Command>(input)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
        let intent: Command =
            serde_json::from_value(json!({"type":"intent","generation":1,"command":create}))
                .unwrap();
        assert!(
            intent.validate().is_err(),
            "account writes cannot share playback intents"
        );
    }

    #[test]
    fn rejects_external_pagination_and_invalid_controls() {
        assert!(
            Command::Library {
                id: Some(9_007_199_254_740_992),
                next: None
            }
            .validate()
            .is_err()
        );
        assert!(
            Command::Library {
                id: Some(0),
                next: None
            }
            .validate()
            .is_ok()
        );
        assert!(
            Command::Library {
                id: None,
                next: Some("https://evil.example".into())
            }
            .validate()
            .is_err()
        );
        assert!(
            Command::Library {
                id: None,
                next: Some("/v1/me/library/songs?offset=100".into())
            }
            .validate()
            .is_ok()
        );
        assert!(Command::Seek { seconds: f64::NAN }.validate().is_err());
        assert!(Command::Volume { value: 1.1 }.validate().is_err());
        assert!(
            Command::Play {
                items: vec![],
                index: 0,
                generation: None,
                order: None,
            }
            .validate()
            .is_err()
        );
    }
    #[test]
    fn occurrence_orders_reject_invalid_indices_and_manual_context_overlap() {
        let order = |upcoming: Vec<usize>, manual_count| QueueOrder {
            upcoming,
            manual_count,
            context: vec![0, 1],
            history: vec![0],
        };
        assert!(order(vec![2, 1], 1).validate(3, Some(0)).is_ok());
        assert!(order(vec![2, 2, 1], 1).validate(3, Some(0)).is_err());
        assert!(order(vec![3, 1], 1).validate(3, Some(0)).is_err());
        assert!(order(vec![0, 1], 1).validate(3, Some(0)).is_err());
        assert!(order(vec![2, 1], 2).validate(3, Some(0)).is_err());
        assert!(order(vec![2, 1], 0).validate(3, Some(0)).is_err());
    }

    fn token(expires: u64) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"ES256","kid":"TEST"}"#);
        let claims =
            URL_SAFE_NO_PAD.encode(json!({"iss":"TEST", "iat":10,"exp":expires}).to_string());
        format!("{header}.{claims}.{}", URL_SAFE_NO_PAD.encode([0u8; 64]))
    }

    #[test]
    fn checks_token_shape_and_expiry_without_claiming_signature_verification() {
        assert!(validate_token_at(&token(200), 100).is_ok());
        assert!(validate_token_at(&token(100), 100).is_err());
        assert!(validate_token_at("-----BEGIN PRIVATE KEY-----", 100).is_err());
        assert!(validate_token_at("fake.fake.fake", 100).is_err());
    }

    #[test]
    fn startup_token_uses_executable_companion_only_when_protected_token_needs_renewal() {
        let root =
            std::env::temp_dir().join(format!("applifast-startup-token-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let executable = root.join("Applifast.exe");
        let path = root.join("developer-token.txt");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let signed = |expires| {
            let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"ES256","kid":"TEST"}"#);
            let claims = URL_SAFE_NO_PAD.encode(
                json!({
                    "iss":"TEST", "iat":now - 20, "exp":expires,
                    "origin":["https://applifast.invalid"]
                })
                .to_string(),
            );
            format!("{header}.{claims}.{}", URL_SAFE_NO_PAD.encode([0u8; 64]))
        };
        let stored = signed(now + 3600);
        let bundled = signed(now + 7200);
        let failure = startup_developer_token(None, &executable).unwrap_err();
        assert!(failure.contains("Download a fresh Applifast build"));
        assert!(!failure.contains(root.to_str().unwrap()));
        std::fs::write(&path, "private fixture contents").unwrap();
        assert_eq!(
            startup_developer_token(Some(stored.clone()), &executable).unwrap(),
            (stored.clone(), false)
        );
        assert_eq!(
            startup_developer_token(None, &executable).unwrap_err(),
            failure
        );
        std::fs::write(&path, format!("\r\n{bundled}\r\n")).unwrap();
        for saved in [
            None,
            Some("invalid stored fixture".into()),
            Some(signed(now - 1)),
        ] {
            assert_eq!(
                startup_developer_token(saved, &executable).unwrap(),
                (bundled.clone(), true)
            );
        }
        assert!(startup_developer_token(None, &root.join("elsewhere/Applifast.exe")).is_err());
        for invalid in [signed(now - 1).into_bytes(), vec![b'x'; 32769], vec![0xff]] {
            std::fs::write(&path, invalid).unwrap();
            assert_eq!(
                startup_developer_token(None, &executable).unwrap_err(),
                failure
            );
        }
        assert_eq!(
            startup_developer_token(Some(stored.clone()), &root.join("missing/Applifast.exe"))
                .unwrap(),
            (stored, false)
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn token_file_is_bounded_private_and_rejects_signing_keys_before_opening() {
        let root =
            std::env::temp_dir().join(format!("applifast-token-read-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("developer-token.txt");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"ES256","kid":"TEST"}"#);
        let claims = |expires| {
            URL_SAFE_NO_PAD.encode(
                json!({
                    "iss":"TEST", "iat":now - 20, "exp":expires,
                    "origin":["https://applifast.invalid"]
                })
                .to_string(),
            )
        };
        let signed = |expires| {
            format!(
                "{header}.{}.{}",
                claims(expires),
                URL_SAFE_NO_PAD.encode([0u8; 64])
            )
        };
        let valid = signed(now + 3600);
        std::fs::write(&path, format!(" \r\n{valid}\r\n ")).unwrap();
        assert_eq!(read_developer_token_file(&path).unwrap(), valid);
        std::fs::write(&path, signed(now - 1)).unwrap();
        assert_eq!(
            read_developer_token_file(&path).unwrap_err(),
            "Developer token expired. Generate and import a fresh token."
        );
        for (bytes, message) in [
            (vec![0xff], "Developer token must be UTF-8."),
            (vec![b'x'; 32769], "Developer token file exceeds 32 KiB."),
            (
                b"private fixture contents".to_vec(),
                "Expected a signed MusicKit developer JWT, not a .p8 signing key.",
            ),
        ] {
            std::fs::write(&path, bytes).unwrap();
            assert_eq!(read_developer_token_file(&path).unwrap_err(), message);
        }
        assert_eq!(
            read_developer_token_file(&root.join("missing.P8")).unwrap_err(),
            "Expected a signed MusicKit developer JWT, not a .p8 signing key."
        );
        assert_eq!(
            read_developer_token_file(&root.join("missing.txt")).unwrap_err(),
            "Cannot open developer token file."
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
