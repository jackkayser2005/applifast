//! Music links as they arrive from outside: the desktop's URL handler,
//! the command line, and a second launch handing one to the running
//! instance.
//!
//! Apple links retain their catalog/library identity. Retained Spotify
//! handling is separate, for legacy callers and deterministic demo coverage.

/// Resource pages. Search links carry text instead of a resource id.
const KINDS: [&str; 6] = ["track", "album", "artist", "playlist", "show", "episode"];

/// Apple share URLs and internal URIs, retaining catalog and library identity.
/// Parsing never follows redirects or changes the authorized storefront.
pub fn parse_apple(text: &str) -> Option<String> {
    let text = text.trim();
    if text.len() > 4096 || text.contains('\\') || text.chars().any(char::is_control) {
        return None;
    }
    if let Some(rest) = text.strip_prefix("apple:") {
        let (kind, identity) = rest.split_once(':')?;
        let (source, id) = identity.split_once('.')?;
        return apple_uri(kind, source, id);
    }
    // URL parsers normalize dot segments. Reject them before normalization.
    for part in text.split(['?', '#']).next()?.split('/') {
        let decoded = percent_encoding::percent_decode_str(part)
            .decode_utf8()
            .ok()?;
        if matches!(decoded.as_ref(), "." | "..")
            || decoded.contains(['/', '\\'])
            || decoded.chars().any(char::is_control)
        {
            return None;
        }
    }
    let url = reqwest::Url::parse(text).ok()?;
    if url.scheme() != "https"
        || url.host_str() != Some("music.apple.com")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return None;
    }
    let segments = url
        .path()
        .trim_end_matches('/')
        .split('/')
        .skip(1)
        .collect::<Vec<_>>();
    if segments.iter().any(|segment| segment.is_empty()) {
        return None;
    }
    let (storefront, kind, id) = match segments.as_slice() {
        [storefront, kind, id] | [storefront, kind, _, id] => (*storefront, *kind, *id),
        _ => return None,
    };
    if storefront.len() != 2 || !storefront.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        return None;
    }
    let mut song_ids = url.query_pairs().filter(|(key, _)| key == "i");
    if let Some((_, song)) = song_ids.next() {
        if kind != "album"
            || song_ids.next().is_some()
            || apple_uri("album", "catalog", id).is_none()
        {
            return None;
        }
        return apple_uri("track", "catalog", &song);
    }
    apple_uri(if kind == "song" { "track" } else { kind }, "catalog", id)
}

fn apple_uri(kind: &str, source: &str, id: &str) -> Option<String> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return None;
    }
    let valid = match (source, kind) {
        ("catalog", "track" | "album" | "artist") => id.bytes().all(|byte| byte.is_ascii_digit()),
        ("catalog", "playlist") => id.starts_with("pl.") && id.len() > 3,
        ("library", "track") => id.starts_with("i.") && id.len() > 2,
        ("library", "album" | "artist" | "playlist") => true,
        _ => false,
    };
    valid.then(|| format!("apple:{kind}:{source}.{id}"))
}

/// Public catalog URL. Private library IDs never become catalog matches.
pub fn public_apple_url(uri: &str, storefront: &str) -> Option<String> {
    let uri = parse_apple(uri)?;
    let (kind, identity) = uri.strip_prefix("apple:")?.split_once(':')?;
    let id = identity.strip_prefix("catalog.")?;
    if storefront.len() != 2 || !storefront.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return None;
    }
    let kind = if kind == "track" { "song" } else { kind };
    Some(format!("https://music.apple.com/{storefront}/{kind}/{id}"))
}

/// The canonical form of a context URI Spotify reports as playing.
/// Personalized playlists report their context with the owner embedded,
/// `spotify:user:NAME:playlist:ID`, while the app's models hold the plain
/// `spotify:playlist:ID`; strict comparisons need the one shape.
pub fn canonical_context_uri(uri: &str) -> String {
    if let Some(rest) = uri.strip_prefix("spotify:user:") {
        const PLAYLIST: &str = ":playlist:";
        if let Some(at) = rest.find(PLAYLIST) {
            let id = &rest[at + PLAYLIST.len()..];
            if !id.is_empty() && !id.contains(':') {
                return format!("spotify:playlist:{id}");
            }
        }
    }
    uri.to_owned()
}

/// The canonical `spotify:<kind>:<id>` behind `text`, or `None` when it is
/// not a link to a track, album, artist, playlist, show, episode, or search.
///
/// Accepted: `spotify:track:ID`, the old `spotify:user:NAME:playlist:ID`,
/// `spotify://track/ID`, and `https://open.spotify.com/track/ID` with or
/// without a locale segment (`/intl-de/`), a query string, or the old
/// `/user/NAME/playlist/ID` shape.
pub fn parse(text: &str) -> Option<String> {
    if let Some(query) = search_query(text) {
        return Some(format!(
            "spotify:search:{}",
            percent_encoding::utf8_percent_encode(&query, percent_encoding::NON_ALPHANUMERIC)
        ));
    }
    let text = text.trim();
    let mut segments: Vec<&str> = if let Some(rest) = text.strip_prefix("spotify://") {
        // The URL shape of the URI: `spotify://track/ID`, or the web
        // address with its scheme swapped.
        let mut segments = path_segments(rest);
        if segments.first().is_some_and(|host| is_web_host(host)) {
            segments.remove(0);
        }
        segments
    } else if let Some(rest) = text.strip_prefix("spotify:") {
        rest.split(':').filter(|part| !part.is_empty()).collect()
    } else {
        let rest = text
            .strip_prefix("https://")
            .or_else(|| text.strip_prefix("http://"))?;
        let mut segments = path_segments(rest);
        if !segments.first().is_some_and(|host| is_web_host(host)) {
            return None;
        }
        segments.remove(0);
        segments
    };
    // Old playlist links carry the owner: spotify:user:NAME:playlist:ID.
    if segments.len() >= 4 && segments[0] == "user" && segments[2] == "playlist" {
        segments.drain(..2);
    }
    // The web address may start with a locale: open.spotify.com/intl-de/…
    if segments
        .first()
        .is_some_and(|first| first.starts_with("intl-"))
    {
        segments.remove(0);
    }
    let [kind, id, ..] = segments.as_slice() else {
        return None;
    };
    let kind = kind.to_ascii_lowercase();
    if !KINDS.contains(&kind.as_str()) || !is_id(id) {
        return None;
    }
    Some(format!("spotify:{kind}:{id}"))
}

/// Decodes a search link once. A path's `+` is literal, not a form-encoded
/// space. Canonical links encode the whole query so delimiters and Unicode
/// survive command-line, D-Bus, Apple Event and line-based socket delivery.
pub fn search_query(text: &str) -> Option<String> {
    let text = text.trim();
    let encoded = if text.starts_with("spotify:") && !text.starts_with("spotify://") {
        let (kind, query) = text.strip_prefix("spotify:")?.split_once(':')?;
        if !kind.eq_ignore_ascii_case("search") {
            return None;
        }
        query
    } else {
        let path = if let Some(rest) = text.strip_prefix("spotify://") {
            match rest.split_once('/') {
                Some((host, path)) if is_web_host(host) => path,
                _ => rest,
            }
        } else {
            let rest = text
                .strip_prefix("https://")
                .or_else(|| text.strip_prefix("http://"))?;
            let (host, path) = rest.split_once('/')?;
            if !is_web_host(host) {
                return None;
            }
            path
        };
        let path = &path[..path.find(['?', '#']).unwrap_or(path.len())];
        let path = if path.starts_with("intl-") {
            path.split_once('/')?.1
        } else {
            path
        };
        let (kind, query) = path.split_once('/').unwrap_or((path, ""));
        if !kind.eq_ignore_ascii_case("search") {
            return None;
        }
        query
    };
    let query = percent_encoding::percent_decode_str(encoded)
        .decode_utf8()
        .ok()?;
    (!query.chars().any(char::is_control)).then(|| query.into_owned())
}

/// The path of a web address split at slashes, its query and fragment
/// dropped, empty segments (a trailing slash) with them.
fn path_segments(rest: &str) -> Vec<&str> {
    let end = rest.find(['?', '#']).unwrap_or(rest.len());
    rest[..end]
        .split('/')
        .filter(|part| !part.is_empty())
        .collect()
}

fn is_web_host(host: &str) -> bool {
    matches!(
        host.to_ascii_lowercase().as_str(),
        "open.spotify.com" | "play.spotify.com"
    )
}

/// Spotify ids are base62; anything else on a link is not one, whatever
/// hands it over.
fn is_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_apple_links_only_share_catalog_ids_in_the_selected_storefront() {
        for (uri, path) in [
            ("apple:track:catalog.123", "song/123"),
            ("apple:album:catalog.456", "album/456"),
            ("apple:artist:catalog.789", "artist/789"),
            ("apple:playlist:catalog.pl.example", "playlist/pl.example"),
        ] {
            let url = public_apple_url(uri, "de").unwrap();
            assert_eq!(url, format!("https://music.apple.com/de/{path}"));
            assert_eq!(parse_apple(&url).as_deref(), Some(uri));
        }
        for uri in [
            "apple:track:library.i.upload",
            "apple:album:library.l.album",
            "apple:artist:library.l.artist",
            "apple:playlist:library.p.playlist",
            "spotify:track:123",
            "apple:track:catalog.123/extra",
        ] {
            assert_eq!(public_apple_url(uri, "us"), None, "{uri}");
        }
        for storefront in ["", "USA", "US", "u/", "é"] {
            assert_eq!(
                public_apple_url("apple:track:catalog.123", storefront),
                None
            );
        }
    }

    #[test]
    fn apple_share_urls_and_internal_ids_remain_distinct() {
        for (link, uri) in [
            (
                "https://music.apple.com/us/album/trying/1616728060?i=1616728064&ls=1",
                "apple:track:catalog.1616728064",
            ),
            (
                "https://music.apple.com/gb/song/a-song/123",
                "apple:track:catalog.123",
            ),
            (
                "https://music.apple.com/de/album/na%C3%AFve/456/",
                "apple:album:catalog.456",
            ),
            (
                "https://music.apple.com/us/artist/789",
                "apple:artist:catalog.789",
            ),
            (
                "https://music.apple.com/us/playlist/name/pl.ab-CD_12",
                "apple:playlist:catalog.pl.ab-CD_12",
            ),
            (
                "apple:track:library.i.upload",
                "apple:track:library.i.upload",
            ),
            (
                "apple:album:library.l.upload",
                "apple:album:library.l.upload",
            ),
            (
                "apple:artist:library.r.artist",
                "apple:artist:library.r.artist",
            ),
            (
                "apple:playlist:library.p.test",
                "apple:playlist:library.p.test",
            ),
        ] {
            assert_eq!(parse_apple(link).as_deref(), Some(uri), "{link}");
            assert_eq!(parse_apple(uri).as_deref(), Some(uri));
        }
    }

    #[test]
    fn apple_links_reject_ambiguous_songs_hosts_and_path_injection() {
        for link in [
            "spotify:track:123",
            "apple:track:catalog.i.upload",
            "apple:track:library.123",
            "apple:track:library.i.upload:catalog.123",
            "apple:album:catalog.123/../456",
            "apple:show:catalog.123",
            "apple:playlist:catalog.123",
            "apple:track:catalog.",
            "http://music.apple.com/us/song/123",
            "https://music.apple.com.evil/us/song/123",
            "https://music.apple.com@evil/us/song/123",
            "https://user:secret@music.apple.com/us/song/123",
            "https://music.apple.com:8443/us/song/123",
            "https://music.apple.com/us/album/name/123?i=456&i=789",
            "https://music.apple.com/us/album/name/123?i=",
            "https://music.apple.com/us/album/name/123?i=i.upload",
            "https://music.apple.com/us/album/name/abc?i=456",
            "https://music.apple.com/us/song/123?i=456",
            "https://music.apple.com/us/album/../song/123",
            "https://music.apple.com/us/%2e%2e/song/123",
            "https://music.apple.com/us/album/a%2Fb/123",
            "https://music.apple.com/us/song/%31%32%33",
            "https://music.apple.com/us/song/123/extra",
            "https://music.apple.com/us/song/12\n3",
            "https:\\music.apple.com/us/song/123",
            "https://music.apple.com/us/song/123?i=456%0A789",
        ] {
            assert_eq!(parse_apple(link), None, "{link:?}");
        }
        assert!(parse_apple(&format!("apple:track:library.i.{}", "x".repeat(127))).is_none());
        assert!(
            parse_apple(&format!(
                "https://music.apple.com/us/song/123?{}",
                "x".repeat(4096)
            ))
            .is_none()
        );
    }

    /// Every shape Spotify's own apps and site hand out lands on the one
    /// URI the app navigates by.
    #[test]
    fn every_link_shape_becomes_the_one_uri() {
        // #given / #when / #then
        for (link, uri) in [
            (
                "spotify:track:4uLU6hMCjMI75M1A2tKUQC",
                "spotify:track:4uLU6hMCjMI75M1A2tKUQC",
            ),
            (
                "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M",
                "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M",
            ),
            (
                "spotify:user:carmine:playlist:37i9dQZF1DXcBWIGoYBM5M",
                "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M",
            ),
            (
                "spotify://album/1DFixLWuPkv3KT3TnV35m3",
                "spotify:album:1DFixLWuPkv3KT3TnV35m3",
            ),
            (
                "spotify://open.spotify.com/artist/4Z8W4fKeB5YxbusRsdQVPb",
                "spotify:artist:4Z8W4fKeB5YxbusRsdQVPb",
            ),
            (
                "https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC",
                "spotify:track:4uLU6hMCjMI75M1A2tKUQC",
            ),
            (
                "https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC?si=abc123&nd=1",
                "spotify:track:4uLU6hMCjMI75M1A2tKUQC",
            ),
            (
                "https://open.spotify.com/intl-it/album/1DFixLWuPkv3KT3TnV35m3/",
                "spotify:album:1DFixLWuPkv3KT3TnV35m3",
            ),
            (
                "https://open.spotify.com/user/carmine/playlist/37i9dQZF1DXcBWIGoYBM5M",
                "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M",
            ),
            (
                "http://play.spotify.com/show/4rOoJ6Egrf8K2IrywzwOMk",
                "spotify:show:4rOoJ6Egrf8K2IrywzwOMk",
            ),
            (
                "https://OPEN.SPOTIFY.COM/episode/512ojhOuo1ktJprKbVcKyQ#top",
                "spotify:episode:512ojhOuo1ktJprKbVcKyQ",
            ),
            (
                "  spotify:Artist:4Z8W4fKeB5YxbusRsdQVPb\n",
                "spotify:artist:4Z8W4fKeB5YxbusRsdQVPb",
            ),
        ] {
            assert_eq!(parse(link).as_deref(), Some(uri), "{link}");
        }
    }

    /// What is not a page here, or not Spotify's at all, is refused rather
    /// than guessed at.
    #[test]
    fn what_is_not_a_page_is_refused() {
        // #given / #when / #then
        for link in [
            "",
            "spotify:",
            "spotify:track",
            "spotify:track:",
            "spotify:user:carmine",
            "spotify:search:bad%0Aquery",
            "spotify:station:track:4uLU6hMCjMI75M1A2tKUQC",
            "spotify:local:Artist:Album:Song:180",
            "spotify:track:4uLU6hMCjMI75M1A2tKUQC/../etc",
            "spotify:track:a b",
            "https://example.com/track/4uLU6hMCjMI75M1A2tKUQC",
            "https://open.spotify.com/",
            "https://open.spotify.com/user/carmine",
            "https://open.spotify.com/intl-it/",
            "file:///etc/passwd",
            "4uLU6hMCjMI75M1A2tKUQC",
        ] {
            assert_eq!(parse(link), None, "{link:?}");
        }
        let long = format!("spotify:track:{}", "x".repeat(65));
        assert_eq!(parse(&long), None);
    }

    #[test]
    fn search_links_preserve_the_query_across_normalization_and_delivery() {
        for (link, query) in [
            (
                "https://open.spotify.com/search/here%20comes%20the%20sun",
                "here comes the sun",
            ),
            (
                "https://open.spotify.com/intl-de/search/artist%3ABj%C3%B6rk?si=share#top",
                "artist:Björk",
            ),
            (
                "spotify:search:artist:Radiohead year:1997",
                "artist:Radiohead year:1997",
            ),
            ("spotify://search/%E6%9D%B1%E4%BA%AC", "東京"),
            ("spotify://open.spotify.com/search/AC%2FDC", "AC/DC"),
            ("http://play.spotify.com/search/C%2B%2B+100%25", "C+++100%"),
            ("spotify:search:%2520", "%20"),
            ("https://open.spotify.com/search", ""),
            ("https://open.spotify.com/search/", ""),
            ("spotify:search:", ""),
        ] {
            assert_eq!(search_query(link).as_deref(), Some(query), "{link}");
            let canonical = parse(link).unwrap();
            assert_eq!(search_query(&canonical).as_deref(), Some(query));
            assert_eq!(parse(&canonical).as_ref(), Some(&canonical));
            assert!(canonical.is_ascii() && !canonical.contains(['\n', ' ']));
        }
        for invalid in [
            "https://example.com/search/song",
            "https://open.spotify.com.evil/search/song",
            "https://user@open.spotify.com/search/song",
            "file:///search/song",
            "spotify:search:bad%FFutf8",
            "https://open.spotify.com/search/line%0Abreak",
            "spotify:search:zero%00byte",
        ] {
            assert_eq!(parse(invalid), None, "{invalid}");
        }
    }
}
