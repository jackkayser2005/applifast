//! The temporary playback probe's input boundary. No Spotify IDs or token logging.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize)]
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
        next: Option<String>,
    },
    Request {
        id: u64,
        path: String,
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
                            | Self::SignOut
                            | Self::Shutdown
                    )
                {
                    return Err("Invalid playback intent.".into());
                }
                command.validate()
            }
            Self::Library { next: Some(next) }
                if !valid_read_path(next) || !next.starts_with("/v1/me/library/songs?") =>
            {
                Err("Only a next-page path for the song library is accepted.".into())
            }
            Self::Request { id, path } if *id > 9_007_199_254_740_991 || !valid_read_path(path) => {
                Err("Only an Apple Music catalog or library read is accepted.".into())
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
            url::form_urlencoded::parse(query.as_bytes()).all(|(key, _)| {
                ["limit", "offset", "term", "types", "include"].contains(&key.as_ref())
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
        ] {
            assert!(!valid_read_path(path), "{path}");
        }
        for path in [
            "/v1/me/library/playlists?limit=100",
            "/v1/me/library/playlists/p.1/tracks?offset=100",
            "/v1/catalog/us/artists/123/view/top-songs?limit=20",
            "/v1/catalog/us/search?term=Some%20song&types=songs,albums",
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
    fn rejects_external_pagination_and_invalid_controls() {
        assert!(
            Command::Library {
                next: Some("https://evil.example".into())
            }
            .validate()
            .is_err()
        );
        assert!(
            Command::Library {
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
}
