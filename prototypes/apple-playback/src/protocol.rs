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
            if !params.is_object() || params.get("id").and_then(Value::as_str) != Some(&self.id) {
                return Err("Playback parameters must retain the selected item's ID.".into());
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
    Play {
        items: Vec<PlaybackItem>,
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        generation: Option<u64>,
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
                            | Self::SignOut
                            | Self::Shutdown
                    )
                {
                    return Err("Invalid playback intent.".into());
                }
                command.validate()
            }
            Self::Library { next: Some(next) }
                if next.len() > 2048 || !next.starts_with("/v1/me/library/songs?") =>
            {
                Err("Only a next-page path for the song library is accepted.".into())
            }
            Self::Play {
                items,
                index,
                generation,
            } => {
                if generation.is_some_and(|value| value > 9_007_199_254_740_991) {
                    return Err("Playback generation exceeds the JavaScript integer range.".into());
                }
                if items.is_empty() || items.len() > 1000 || *index >= items.len() {
                    return Err("Choose an existing row in a queue of 1 to 1000 songs.".into());
                }
                items.iter().try_for_each(PlaybackItem::validate)
            }
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
            }
            .validate()
            .is_err()
        );
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
