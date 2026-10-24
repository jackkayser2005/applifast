# Favorite metadata in collections and search

Baseline: `465a5bb547afa0afa7ff8d84b126bf822468df9b` (PR #33).
Windows validation: October 10, 2026.

## Problem and change

The Songs library request already asked for Apple's extended `inFavorites`
attribute. Playlist, album, artist, search and Home requests did not. Song
resources from those responses could therefore reach the queue or player with
unknown favorite status, even though the existing read-only hearts worked.

The shared MusicKit read helper now adds `extend[songs]=inFavorites` and
`extend[library-songs]=inFavorites`. Apple's documentation allows extending a
resource type throughout a response, including nested relationships. Both
song attribute dictionaries document `inFavorites`:

- [Resource representations and relationships](https://developer.apple.com/documentation/applemusicapi/handling-resource-representation-and-relationships)
- [Catalog song attributes](https://developer.apple.com/documentation/applemusicapi/songs/attributes-data.dictionary)
- [Library song attributes](https://developer.apple.com/documentation/applemusicapi/librarysongs/attributes-data.dictionary)

Direct library-song reads retain their existing unscoped extension and album/
artist relationships. Original search terms, collection IDs, pagination and
relationship selections survive. The native allowlist accepts these two scoped
keys only with `inFavorites`, including Apple's encoded pagination URLs.
Playlist POST requests are unchanged.

This changes existing requests to `api.music.apple.com`, not their count or
destination. No dependencies, credential handling, saved formats, audio paths,
interface drawing or layouts change. Responses retain original playback
parameters and library/catalog IDs. No catalog-match or ISRC inference is used.
Missing favorite flags remain unknown, and the current library snapshot still
takes precedence over older queue metadata. Favoriting and unfavoriting remain
outside this read-only slice.

## Verification

The Node regression failed against the baseline's collection reads and passes
with the shared-helper change. It checks scoped flags, original parameters,
mixed catalog/upload metadata, unknown values and unchanged playlist writes.
Native protocol regressions reject other scopes, attributes and credential
parameters. The app regression carries explicit true/false/unknown flags from
collection pages into exact rows and duplicate queue occurrences, preserving
the uploaded song ID and pagination.

The opt-in account test compares baseline and extended read responses using
protected local credentials. It permits only approved HTTPS Apple GET routes,
checks authorization before and after requests and prints aggregate counts
only. It does not write playlists/favorites, launch playback, alter credentials
or print song names, resource IDs or authorization responses.

The read-only account comparison passed:

| Route | Song resources | Explicit flags before | Explicit flags after |
| --- | ---: | ---: | ---: |
| Library playlist tracks | 14 | 0 | 14 |
| Library playlist embedded tracks | 14 | 0 | 14 |
| Library album tracks | 2 | 0 | 2 |
| Library album embedded tracks | 2 | 0 | 2 |
| Catalog search | 5 | 0 | 5 |
| Library search | 1 | 0 | 1 |

Each comparison retained the same resource count, IDs, types and playback
parameters. The extended Home requests also loaded: 10 Recent, 10 Added,
6 Heavy Rotation and 10 Recommendations resources (64 recommendation cards).
The first 100 library albums retained 100 parsed add dates. The first 100
library songs retained their identities and playback parameters and supplied
100 explicit favorite flags, including 30 favorites.

These are sampled responses from the existing authorized account, not a full
library audit or a fresh-listener sign-in test. Standalone host formatting,
strict Clippy and all-target tests passed (13 tests, one native-store check
ignored). App formatting, strict default/demo all-target Clippy, 1,009 default
and 1,043 demo library tests (four opt-in checks ignored in each), binary and
integration suites, default doctests, strict demo Rustdoc, the demo build,
gettext and Node bridge/signer checks passed. Portable-package fixtures passed
with dummy JWTs. README/tester-guide links, issue-form parsing and this report's
local links passed. Credential storage did not change; its native round trip
was already verified in PR #33 and was not repeated here.

Runtime playback, fresh-user sign-in, gaming resource measurements, Linux/macOS
compilation, optional projectM, Jekyll and Nix remain separate acceptance gates.

There is no new visual scope. The matching Windows light/dark and narrow/normal
[heart captures](review-favorite-hearts/index.html) and
[sign-in captures](review-bundled-token/index.html) continue to describe the UI.
API metadata checks do not substitute for runtime playback acceptance.
