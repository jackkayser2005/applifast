//! Apple resources mapped into the existing desktop display models.
use super::Song;
use crate::api::models::{Album, Artist, ArtistRef, Image, Owner, Playlist, TrackCount};
use applifast_playback_probe::protocol::{ItemKind, PlaybackItem};
use serde_json::Value;

pub fn added_at(resource: &Value) -> Option<String> {
    let date = resource["attributes"]["dateAdded"].as_str()?;
    let date = match date.len() {
        4 => format!("{date}-01-01T00:00:00Z"),
        10 => format!("{date}T00:00:00Z"),
        _ => date.to_owned(),
    };
    date.parse::<jiff::Timestamp>().ok().map(|_| date)
}

pub fn home_card(resource: &Value) -> Option<super::HomeCard> {
    use crate::model::Page;
    let (name, subtitle, image, uri, page, playable) = match resource["type"].as_str()? {
        "songs" | "library-songs" => {
            let song = song(resource)?;
            let page = song.album_id.clone().map(Page::Album);
            (
                song.title.clone(),
                song.artist.clone(),
                song.artwork.clone(),
                song.uri(),
                page,
                song.available(),
            )
        }
        "albums" | "library-albums" => {
            let album = album(resource);
            (
                album.name,
                album
                    .artists
                    .iter()
                    .map(|artist| artist.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                album.images.first().map(|image| image.url.clone()),
                album.uri,
                Some(Page::Album(album.id)),
                true,
            )
        }
        "playlists" | "library-playlists" => {
            let playlist = playlist(resource);
            let owner = playlist.owner_name().to_owned();
            (
                playlist.name,
                owner,
                playlist.images.first().map(|image| image.url.clone()),
                playlist.uri,
                Some(Page::Playlist(playlist.id)),
                true,
            )
        }
        "artists" | "library-artists" => {
            let artist = artist(resource);
            (
                artist.name,
                String::new(),
                artist.images.first().map(|image| image.url.clone()),
                artist.uri,
                Some(Page::Artist(artist.id)),
                false,
            )
        }
        _ => return None,
    };
    Some(super::HomeCard {
        name,
        subtitle,
        image,
        uri,
        page,
        playable,
    })
}

fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_owned()
}
pub fn id(resource: &Value) -> String {
    format!(
        "{}.{}",
        if resource["type"]
            .as_str()
            .is_some_and(|kind| kind.starts_with("library-"))
        {
            "library"
        } else {
            "catalog"
        },
        text(resource, "id")
    )
}
fn related_id(resource: &Value, relationship: &str) -> Option<String> {
    resource["relationships"][relationship]["data"]
        .as_array()?
        .first()
        .map(id)
}
fn images(resource: &Value) -> Vec<Image> {
    resource["attributes"]["artwork"]["url"]
        .as_str()
        .into_iter()
        .map(|url| Image {
            url: url.replace("{w}", "640").replace("{h}", "640"),
            width: Some(640),
            height: Some(640),
        })
        .collect()
}
fn artists(resource: &Value) -> Vec<ArtistRef> {
    let artist_id = related_id(resource, "artists");
    vec![ArtistRef {
        id: artist_id.clone(),
        name: text(&resource["attributes"], "artistName"),
        uri: artist_id.map(|id| format!("apple:artist:{id}")),
    }]
}
pub fn song(resource: &Value) -> Option<Song> {
    let attributes = &resource["attributes"];
    let kind = match resource["type"].as_str()? {
        "library-songs" => ItemKind::Library,
        "songs" => ItemKind::Catalog,
        _ => return None,
    };
    let item = PlaybackItem {
        kind,
        id: text(resource, "id"),
        play_params: attributes
            .get("playParams")
            .filter(|value| value.is_object())
            .cloned(),
    };
    item.validate().ok()?;
    Some(Song {
        catalog_id: item
            .play_params
            .as_ref()
            .and_then(|params| params["catalogId"].as_str().map(str::to_owned)),
        item,
        title: text(attributes, "name"),
        artist: text(attributes, "artistName"),
        album: text(attributes, "albumName"),
        duration_ms: attributes["durationInMillis"]
            .as_u64()
            .unwrap_or(0)
            .min(u64::from(u32::MAX)) as u32,
        artwork: images(resource).first().map(|image| image.url.clone()),
        album_id: related_id(resource, "albums"),
        artist_id: related_id(resource, "artists"),
        in_favorites: attributes["inFavorites"].as_bool(),
    })
}
pub fn album(resource: &Value) -> Album {
    let attributes = &resource["attributes"];
    let id = id(resource);
    Album {
        uri: format!("apple:album:{id}"),
        id,
        name: text(attributes, "name"),
        artists: artists(resource),
        images: images(resource),
        album_type: Some(
            if attributes["isCompilation"] == true {
                "compilation"
            } else if attributes["isSingle"] == true {
                "single"
            } else {
                "album"
            }
            .into(),
        ),
        total_tracks: attributes["trackCount"]
            .as_u64()
            .map(|count| count.min(u64::from(u32::MAX)) as u32),
        release_date: attributes["releaseDate"].as_str().map(str::to_owned),
        ..Default::default()
    }
}
pub fn artist(resource: &Value) -> Artist {
    let id = id(resource);
    Artist {
        uri: format!("apple:artist:{id}"),
        id,
        name: text(&resource["attributes"], "name"),
        images: images(resource),
        ..Default::default()
    }
}
pub fn playlist(resource: &Value) -> Playlist {
    let id = id(resource);
    let attributes = &resource["attributes"];
    Playlist {
        uri: format!("apple:playlist:{id}"),
        id,
        name: text(attributes, "name"),
        public: attributes["isPublic"].as_bool(),
        images: images(resource),
        description: attributes["description"]["standard"]
            .as_str()
            .map(str::to_owned),
        owner: Owner {
            display_name: Some(
                attributes["curatorName"]
                    .as_str()
                    .filter(|name| !name.is_empty())
                    .unwrap_or("Apple Music")
                    .into(),
            ),
            ..Default::default()
        },
        tracks: attributes["trackCount"].as_u64().map(|total| TrackCount {
            total: total.min(u64::from(u32::MAX)) as u32,
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn apple_added_dates_keep_timestamp_date_and_year_ordering() {
        for (input, expected) in [
            ("2026-10-08", Some("2026-10-08T00:00:00Z")),
            ("2025", Some("2025-01-01T00:00:00Z")),
            ("2026-10-08T16:30:00Z", Some("2026-10-08T16:30:00Z")),
            ("2026-99-99", None),
            ("unknown", None),
        ] {
            assert_eq!(
                added_at(&json!({"attributes":{"dateAdded":input}})).as_deref(),
                expected
            );
        }
    }
    #[test]
    fn favorites_are_optional_booleans_and_survive_metadata_serialization() {
        for (flag, expected) in [
            (json!(true), Some(true)),
            (json!(false), Some(false)),
            (Value::Null, None),
            (json!("true"), None),
        ] {
            let resource = json!({"id":"i.upload","type":"library-songs","attributes":{
                "name":"Upload", "inFavorites":flag, "rating":1}});
            let song = song(&resource).unwrap();
            assert_eq!(song.in_favorites, expected);
            assert!(!song.available());
            let mut cached = serde_json::to_value(&song).unwrap();
            assert_eq!(
                serde_json::from_value::<Song>(cached.clone())
                    .unwrap()
                    .in_favorites,
                expected
            );
            cached.as_object_mut().unwrap().remove("inFavorites");
            assert_eq!(
                serde_json::from_value::<Song>(cached).unwrap().in_favorites,
                None
            );
        }
    }
    #[test]
    fn library_and_catalog_identity_never_collapse() {
        let upload = json!({"id":"i.upload","type":"library-songs","attributes":{"name":"Upload","playParams":{"id":"i.upload","isLibrary":true,"catalogId":"123"}},"relationships":{"albums":{"data":[{"id":"l.album","type":"library-albums"}]}}});
        let song = song(&upload).unwrap();
        assert_eq!(song.uri(), "apple:track:library.i.upload");
        assert_eq!(song.catalog_id.as_deref(), Some("123"));
        assert_eq!(song.track().album.unwrap().id, "library.l.album");
        assert!(
            !super::song(&json!({"id":"i.upload","type":"library-songs","attributes":{}}))
                .unwrap()
                .available()
        );
        assert_eq!(super::song(&json!({"id":"123","type":"songs","attributes":{"playParams":{"id":"123","kind":"song"}}})).unwrap().uri(), "apple:track:catalog.123");
        let canonical=super::song(&json!({"id":"123","type":"songs","attributes":{"playParams":{"id":"456","kind":"song"}}})).unwrap();
        assert_eq!(canonical.item.id, "123");
        assert_eq!(canonical.item.play_params.unwrap()["id"], "456");
    }
}
