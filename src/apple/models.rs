//! Apple resources mapped into the existing desktop display models.
use super::Song;
use crate::api::models::{Album, Artist, ArtistRef, Image, Owner, Playlist, TrackCount};
use applifast_playback_probe::protocol::{ItemKind, PlaybackItem};
use serde_json::Value;

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
        images: images(resource),
        description: attributes["description"]["standard"]
            .as_str()
            .map(str::to_owned),
        owner: Owner {
            display_name: attributes["curatorName"].as_str().map(str::to_owned),
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
