# First Windows listening slice

Applifast reuses the original Rust/egui interface and an independent MusicKit JS v3 host.
The host uses the installed Evergreen WebView2 runtime on its own COM STA thread
with a message pump. Commands and sanitized events cross channels through the
existing asynchronous backend. Closing an egui window does not own or destroy
the playback host. Apple authorization is the only visible browser popup.

## Preview product identity

Windows executable properties now identify **Applifast** and **Apple Music for
Windows**. The application and HTTP user agent use Applifast; no new network
destination or telemetry is added. The Cargo package/target and `--version`
retain `spotifast 0.12.0` until the packaging/version milestone. Icons and
inherited installer/release metadata are still pending.

The current run writes `applifast.log` in the existing Applifast data directory.
Main-window geometry and egui memory use `window.ron` in that same directory,
instead of upstream's eframe profile. The first launch starts with fresh
main-window geometry and zoom. No old profile or log is moved, copied or deleted;
settings, grants, the account snapshot and mini-player preferences keep their
paths. Demo storage remains isolated. The README and issue forms now describe
Apple preview setup, current capabilities and redacted diagnostics rather than
directing testers to upstream Spotify downloads.

The existing window tests cover independent main-window storage, untouched demo
and mini-player storage and matching main/mini app identity. The local HTTP proxy
test checks the actual outgoing Applifast user-agent header. See the matching
[Windows Home captures](review-product-identity/index.html). These do not prove
first-run/restart geometry, taskbar grouping, fresh-account authorization or
public-release acceptance. Linux/macOS remain unsupported for Apple playback.

The Windows identity slice passes formatting, strict default/demo all-target
Clippy, 992 default and 1,023 demo library tests (four opt-in checks ignored in
each), the binary/integration suites, default doctests, strict demo Rustdoc,
the demo build, gettext and Node bridge/signer checks. Native Windows executable
properties report Applifast / Apple Music for Windows. All eight Home captures
were inspected; comparison selector paths and PNG dimensions pass. Issue forms
parse as YAML and README/tester-guide local links exist. HTML browser rendering,
fresh-account/runtime acceptance, optional projectM/vcpkg, Ruby/Bundler, Nix and
other-platform compilation remain unverified in this slice.

## Supported Apple controls

Apple song, selection, album, artist and playlist menus now hide unavailable
favorite/save/follow writes, Spotify radio and advanced playlist edits. Header
save/follow buttons remain hidden until Apple writes are implemented. Known
Apple favorites display filled, read-only hearts in song rows and the player
bar, using the exact library/catalog identity and Apple's `inFavorites` flag.
Unknown and explicitly non-favorite songs stay unmarked. The current library
snapshot takes precedence over older cached queue/search metadata. Favorites
also remain readable through the existing page and sidebar shelf. See the
[heart comparison](review-favorite-hearts/index.html). Playlist creation and
appending songs/albums/queue remain supported.
The heart follow-up passes formatting, strict default/demo Clippy, 1,006 default
and 1,040 demo library tests, binary/integration suites, default doctests, strict
demo Rustdoc, the build, gettext and Node bridge/signer checks. All 16 matching
Windows heart captures were inspected. Real-account refresh and HTML browser
rendering remain unchecked; the launcher checker requires unavailable Ruby.
Apple playlists keep Copy and selection, while Cut, Paste and Delete cannot
dispatch unsupported edits. Old/direct unsupported actions produce a clear
preview error instead of opening legacy pages or Spotify editing dialogs.

The sidebar omits Podcasts and Spotify custom order in Apple mode. A stale
podcast filter returns to Playlists; an old Spotify sort preference falls back
without erasing the saved preference. Library drag targets accept editable
playlists, but no longer imply that dropping onto Songs changes favorites.

Copy link and Copy songs preserve Apple identity. Catalog items receive public
`https://music.apple.com/<storefront>/<kind>/<id>` links in the authorized
storefront. Private library items retain internal Apple URIs, without using a
catalog match or leaking an authorization token. Open in Apple Music is offered
only for public catalog items and opens that URL in the user's browser on click.
This adds user-initiated browser access to `music.apple.com`; parsing/copying
remains local. There is no new dependency, credential, persisted setting or
background network request. Retained Spotify-mode callers keep their own links.

Normal CLI help hides unsupported Like, Devices, Transfer and device-name
controls. Explicit invocation exits with code 2 and a fixed explanation before
contacting an existing instance. The supported transport commands are unchanged.
The Linux private-bus fixture now uses Apple links and Mute; its runtime remains
unverified on this Windows host.

Focused tests cover outgoing link identity, clipboard/browser commands, guarded
actions, menu accessibility, retained sorting preferences, playlist keyboard
guards, stale sidebar filters and CLI rejection. See the matching
[Windows album, artist and playlist comparison](review-supported-controls/index.html).
These checks do not establish real-account writes or release acceptance.

This slice passes formatting, strict default/demo all-target Clippy, 1,005
default and 1,038 demo library tests (four opt-in checks ignored in each),
binary/integration suites, default doctests, strict demo Rustdoc, the build,
gettext and Node bridge/signer checks. The new browser label is translated in
all 14 complete catalogs. All 24 native captures were inspected; comparison
paths and PNG dimensions pass. HTML browser rendering, real-account acceptance,
optional projectM, site/Nix and non-Windows compilation remain unchecked.

Two earlier runs exposed intermittent existing Windows file-save failures:
the cache-write test failed once, and the session test left the new JSON in
its temporary file while retaining the prior session. Twenty isolated session
trials and the final full default/demo suites passed. The underlying atomic
replacement failure has not been diagnosed; these passing reruns do not prove
that persistence is reliable under all Windows file-lock conditions.

## Local setup

Invited testers: follow the [Windows setup and troubleshooting guide](testing.md).
Maintainers: follow the [developer-token setup](../../prototypes/apple-playback/README.md).
Keep the `.p8` key and generated JWT in ignored `.secrets/apple-music/`, never
in a commit or chat. The app imports the JWT file, not the signing key. On
Windows, run `cargo run --locked`. The executable is still named `spotifast.exe`
during this development slice; its window and local identity are Applifast.

Tester ZIPs can now include `developer-token.txt` next to `Applifast.exe`.
The host keeps a valid app token from Credential Manager; if it is missing or
expired, it reads only that executable's companion file using the existing
bounded JWT validator, then saves it to Credential Manager. File/store work
remains off the UI thread. Nothing is fetched from a token server. Click
**Sign in with Apple** and enter subscriber credentials only in Apple's popup.
The sign-in card hides manual token controls when ready; errors and Settings
retain protected manual import. Existing user grants restore as before, while
the sign-out marker continues to prevent restoring a revoked user session.

The maintainer's [Windows packager](../../packaging/windows/applifast-preview.ps1)
includes only the executable, license, docs, build metadata and signed app JWT.
It rejects `.p8` keys, invalid/expired/oversized files and wrong origins; output
must be Git-ignored. A validation copy is removed on success or failure. ZIP
checksums and the token expiry are recorded without printing token contents.
The local signer still creates 30-day tokens. Expiry requires a new preview or
protected manual renewal. No signing key, listener token, new dependency,
network destination or settings format is added. Public release distribution
and clean-Windows-account authorization acceptance remain pending.

The bundled-token follow-up passes formatting, strict default/demo Clippy,
1,006 default and 1,040 demo library tests, binary/integration suites, default
doctests, strict demo Rustdoc, the build, gettext and Node bridge/signer checks.
The standalone host passes 13 ordinary tests, strict Clippy and its isolated
dummy native Credential Manager round trip. Packager checks reject invalid
inputs, missing/extra origins and unignored output, preserve existing archives,
and verify the whitelist, token content and checksum using dummy tokens only.
All 16 matching [sign-in captures](review-bundled-token/index.html) were inspected.
On October 10, 2026, Apple accepted the existing app JWT for a public catalog
search. This does not prove a fresh listener's authorization popup or playback.
HTML browser rendering, optional projectM, launcher/site tooling, Nix and
non-Windows coverage remain unchecked; earlier persistence failures remain open.

Token selection and bounded local validation run asynchronously before resetting
the host. Cancelled selection, unreadable/expired/invalid files and late results
after sign-out leave the current session untouched. Import and sign-in controls
prevent overlapping setup requests. The sign-in card leaves enough room for
token controls, loading and errors. A current accepted token restarts the host;
the host validates the file again before writing credentials. A changed file or
Apple-rejected signature can still fail that final step. No JWT appears in UI
events, no signing key is imported, and `.p8` paths are rejected before opening.
This adds no dependency, network destination or settings format. Clean-machine
authorization acceptance remains pending.

The token regressions use dummy data: bounded/expired/non-UTF-8 files, signing-key
path rejection, cancelled selection, request generations, delayed success after
sign-out and disabled setup controls. Windows default/demo tests cover the
native sign-in card; its UI regression is Windows-only because other targets
show unsupported playback. The standalone host's dummy Credential Manager
round trip also passes. See the [40 matching native setup captures](review-token-onboarding/index.html).
Browser rendering of that HTML remains unchecked due to local plugin file
permissions. Native picker interaction and fresh-account Apple authorization
remain separate runtime gates.

Available Windows checks pass: formatting, strict default/demo all-target Clippy,
992 default and 1,023 demo library tests (four opt-in checks ignored in each),
binary/integration tests, doctests, strict demo Rustdoc, gettext and Node checks.
The standalone host passes 12 tests plus its opt-in dummy credential-store test.
One preceding demo run had a filesystem error in the existing protected-grant
restoration test. It did not reproduce in 20 isolated repetitions or the final
full run; its cause is unresolved and no test or credential handling was weakened.
Optional projectM/vcpkg, Ruby/Bundler, Nix and other-platform checks remain pending.

## Shared Apple Music links

Paste a `https://music.apple.com/` song, album, artist or playlist link into
Search and press Enter to open it. Launching the app with the quoted link does
the same, forwarding to an existing instance or waiting for Apple authorization.
Opening a link navigates without starting playback. A song opens its album when
Apple provides that relationship. `play-uri` explicitly starts playback instead:

```powershell
cargo run --locked -- "https://music.apple.com/us/album/trying/1616728060?i=1616728064"
cargo run --locked -- play-uri "https://music.apple.com/us/album/trying/1616728060?i=1616728064"
```

Internal `apple:track:library.i.…` and `apple:track:catalog.…` identifiers also
work. Library identity and Apple's original playback parameters remain intact;
a library song is never resolved by substituting a catalog ID. Album links with
an `i` query select that song, matching [Apple's documented share URLs](https://developer.apple.com/documentation/applemusicapi/get-a-catalog-album).
The authorized account's storefront is used, even when the shared URL names a
different country. Availability can differ. Shortened links, stations, videos,
HTTP URLs and automatic OS URL-handler registration are outside this slice.

Unknown songs request only their exact Apple song resource and album/artist
relationships through the existing asynchronous MusicKit read channel. Opening
does not depend on a Spotify account profile. Duplicate reads are coalesced;
newer links or navigation supersede older navigation, and newer playback or
transport actions cancel pending playback. Failed or mismatched song responses
leave existing playback and queue occurrences intact. Sign-out/cancellation
clear pending links and reads; late answers cannot revive them. A developer-token
renewal retains the pending navigation until authorization succeeds.

Input validation bounds identifiers, refuses ambiguous song parameters and
path/host injection, and never follows a redirect. Rejected input is not echoed
to logs or the terminal. No dependency, storage format, credential entry,
network destination, hosted service or telemetry is added. Retained Spotify
parsing remains available to legacy internal callers; the normal CLI accepts
Apple links. The interface layout and styling are unchanged.

Parser, control-channel, exact-song failure, authorization, stale-response and
queue-preservation regressions cover these rules. The demo test submits a link
through the actual Search field with Enter, then checks ordinary text search.
Windows formatting, strict default/demo all-target Clippy, 999 default and
1,031 demo library tests (four opt-in checks ignored in each), binary/integration
suites, default doctests, strict demo Rustdoc, demo build, gettext and Node
bridge/signer checks pass. The standalone host's isolated dummy Credential
Manager round trip passes. All eight matching native album captures were
inspected; selector paths and PNG dimensions pass. See the
[shared-links comparison](review-links/index.html). The baseline opens the
album directly, while the candidate reaches it through the library-song link.
Fixtures retain some inherited Spotify labels; captures do not establish live
account behavior. Real-account link playback is pending while an older app
owns the playback profile. HTML browser rendering, optional projectM/vcpkg,
Ruby/Bundler, Nix and other-platform compilation remain unverified.

## Keyboard shortcuts

Open **Settings > Keyboard shortcuts** beside the page heading, or press
**Ctrl+/**. The menu scrolls in smaller windows and lists the bindings supported
in Apple mode. They operate in the focused app window; navigation changes the
main page, and media keys remain the existing desktop integration. New
navigation and refresh keys wait while a
text field or a dialog is active. Refresh applies to music pages, not Settings
or the local Queue.

| Keys on Windows | Action |
| --- | --- |
| Alt+1 / Alt+2 / Alt+3 | Home / Songs / Favorites |
| Alt+4 / Alt+5 | Albums / Artists |
| F5 or Ctrl+R | Refresh the current music page |
| Ctrl+Shift+Q or Q | Show or hide the queue |
| Space | Play or pause |
| Ctrl+Left / Ctrl+Right | Previous / Next |
| Shift+Left / Shift+Right | Seek backward / forward 10 seconds |
| Ctrl+Up / Ctrl+Down | Volume up / down |
| M / S / R | Mute / Shuffle / Cycle repeat |
| Ctrl+M | Toggle the mini player |
| L or Ctrl+Shift+K | Now Playing |
| Esc | Close the dialog or leave Now Playing |
| Ctrl+, / Ctrl+F | Settings / Search |

Apple mode omits the unimplemented favorite-toggle, playlist removal, cut and
paste bindings from this menu. The B favorite shortcut no longer emits an
unsupported Apple action. Other existing shortcuts remain available. No global
hotkey registration, new preference, storage or network access is added.
Windows keyboard and accessible-button tests cover navigation, refresh,
text/dialog guards and opening the menu without scrolling. See the matching
[Settings and shortcut comparison](review-shortcuts/index.html).
Available Windows checks pass: formatting, strict default/demo all-target
Clippy, 989 default and 1,020 demo library tests (four opt-in checks ignored in
each), binary/integration tests, doctests, strict demo Rustdoc, gettext and
Node bridge/token checks. Optional projectM/vcpkg, Ruby/Bundler site and launcher
checks, Nix and other-platform coverage remain pending. These keyboard checks
do not establish fresh-account authorization, real playback latency or release
acceptance.

## Click-to-play preparation

Batch song resolution indexes the loaded library once per operation instead of
scanning it and constructing every URI again for every queue occurrence. Play,
bulk queue additions and playlist saves share the lookup. Single-song requests
retain the existing direct lookup. Known songs still take precedence over the
library, which takes precedence over the queue; the first occurrence in each
list wins. Library and catalog IDs, original playback parameters, duplicates,
manual queue order and atomic failures are preserved. No persistent index,
audio cache, credential or network destination is added.

Run `cargo run --locked --example apple-click-latency` for a synthetic local
measurement. On this Windows PC in the debug profile, five 1,000-song preparations
(including command serialization) took 63.32–64.85 ms with the preceding
`01c4a3a` lookup, and 6.40–8.25 ms with the batch lookup. At 100 songs, the ranges
were 1.16–1.36 ms and 0.71–0.79 ms. This measures UI-side preparation only, not
MusicKit loading, network time, audible playback latency or release performance.
The focused regression compares lookup precedence, preserves uploaded playback
parameters and duplicate identities, and keeps the queue unchanged on missing
items. Actual first-play and song-switch listening remain runtime checks.

## Read-only account diagnostics

On Windows, this opt-in check reads metadata using Applifast's own saved grants,
without opening a second WebView2 host or interrupting music:

```powershell
cargo test --locked --lib app::apple::tests::account_api_reads_home_and_library -- --ignored --exact --nocapture
```

It sends HTTPS GET requests to `api.music.apple.com` for the four Home feeds,
the first 100 library albums and the first 100 library songs. Requests use the
documented [developer and Music User Token headers](https://developer.apple.com/documentation/applemusicapi/user-authentication-for-musickit).
Redirects are disabled; authorization headers are marked sensitive. It checks
the app's sign-out marker and unchanged saved grants before and after requests,
and after reading JSON. Errors are fixed diagnoses or numeric HTTP statuses.
Output contains aggregate counts only, never tokens, resources or library names.
It writes no credentials, browser profile, account cache or library data. The
ordinary test suite skips this check.

The October 8, 2026 Windows account run passed: Recent returned 10 resources
and 10 rendered cards, Recently added returned 10 and 10, Heavy rotation returned
8 and 8, and Recommendations returned 10 groups and 64 rendered cards. All 100
sampled albums supplied parseable add dates. All 100 sampled library songs
retained their original IDs and playback parameters and supplied explicit
favorite flags; 30 were favorites. This verifies actual API data and the app's
Home parser, not the MusicKit bridge, visible window, complete library,
pagination, favorite writes or playback. The older `3a87e21` preview predates
the Home integration. Quit it through the tray, run the current combined preview,
and use **Home > Refresh** to check the visible feed.

This diagnostic adds no production code, dependency, interface or storage-format
change. Windows formatting and strict default/demo all-target Clippy pass.
The all-target suites pass with 985 default and 1,016 demo library tests, four
opt-in checks skipped in each, plus the binary and integration suites. Default
doc tests, strict demo Rustdoc, the demo build, gettext, bridge and token-generator
self-checks pass. Optional projectM/all-features,
launcher/site/Nix, hosted CI and non-Windows runtime checks retain their
previously documented limitations.

## Supported now

### Library request lifetime

Every desktop song-library page now has its own request ID, including initial
sign-in, cached-session refresh, manual refresh and continuation pages. The
bridge and native sanitizer retain it. Only the current request may replace
loaded songs or finish loading; stale successes, stale failures, duplicate pages
and untagged replies are ignored. Sign-out invalidates the active request.
The existing browser-console diagnostic remains compatible with untagged reads.
Storage and cached song formats are unchanged.

Library reads no longer share the transport command chain. A slow read cannot
hold Pause behind a network response. All MusicKit metadata reads supply a
20-second abort deadline, and sign-out aborts pending API reads as well as writes.
The shared request helper keeps playlist writes' existing confirmation behavior
without adding a timeout that could misreport an already-applied mutation.
Library failures show a fixed retry diagnosis, keep loaded songs and the queue,
and do not pause playback. Playback errors still pause playback but no longer
cancel an independent library refresh. No layout, control, credential store,
cache format, dependency or network destination changes.

Focused fixtures verify out-of-order successes and errors, missing IDs,
duplicate replies, refreshed favorite metadata, unchanged queue occurrences,
playback failure during refresh and sign-out. Bridge checks cover an unresolved
read alongside Pause, a controlled read deadline, fixed error output and aborting
library/Home reads during sign-out without accepting late results. Native tests
verify ID/error sanitization and the JavaScript-safe request-ID boundary.
These checks do not establish actual SDK deadline/cancellation behavior or
integrated listening. Those remain separate Windows account acceptance checks.

Windows checks pass: 986 default and 1,017 demo library tests, with four opt-in
checks skipped in each, binary/integration suites, formatting, strict default/demo
Clippy, default doc tests, strict demo Rustdoc and the demo build. All 11
standalone host tests pass (one native-store check skipped), with standalone
Clippy/formatting. Bridge, token-generator and gettext checks pass. Translation
updates change source references and template dates only, without new messages
or translations. The existing all-features/projectM, launcher/site/Nix, hosted
CI and non-Windows limitations remain. Existing visual comparisons apply because
the views and fixture rendering are unchanged.

### Listening and browsing

- Reuse the original sidebar, tables, artwork, search, account menu and player bar.
  Apple authorization occupies the existing sign-in card. No replacement app shell
  or palette redesign is included. The synced song shelf is called **Songs**.
- Load synced library songs in validated 100-song pages and filter loaded rows.
- Browse library albums, artists and playlists, including collection detail pages
  and paginated tracks. Catalog and library search share the existing search view;
  library and catalog resources retain distinct identities. Catalog artist pages
  request Apple's top-songs view; library artists show their library albums.
  Search offers All, Songs, Artists, Albums and Playlists in Apple mode. Load more
  requests the next library and catalog pages for the selected filter. All requests
  each available resource type. A later-page failure keeps successful rows and
  its cursor for retry; changing the query or signing out invalidates old replies.
  Overlapping pages deduplicate the same resource URI, never a library/catalog
  pair. Unavailable uploads remain visible with their original identity.
- Double-click a playable song to start the loaded-song context; play/pause,
  seek, volume, mute, next, previous, shuffle and repeat use the same action path
  as desktop media keys. Collection cards load their songs before starting playback.
- Preserve original library IDs and Apple's playback parameters, separately
  from catalog IDs. Duplicate occurrences remain distinct in the playback queue.
- Keep unavailable songs visible. A playback failure retains the queue and
  shows an error rather than silently skipping the song.
- Hold optimistic song, pause and seek state against older playback events.
  Sign-out invalidates pending account responses, clears in-memory account data,
  deletes the saved user token, and requests browser-profile clearing.

The first context contains at most 1,000 loaded songs, the host's validated queue
limit. Collection playback currently uses the loaded prefix; complete context
loading and streaming pagination need the queue slice. The existing queue view
shows upcoming occurrences, including repeated songs. Manual queue additions,
album additions, insertion, reordering, removal and clear use the local Apple
occurrence queue. Library/queue restoration retains the current occurrence,
position, manual additions, context, Previous history, shuffle and repeat.
It restores paused. Favorite writes remain follow-up integration work. Playlist creation and appending
are implemented below, with real-account write acceptance pending.
The existing mini player uses the same playback actions, but its window lifecycle
still needs real runtime acceptance. Cloud-only upload playback
remains unverified; no upload is silently replaced with a catalog match.
The player bar omits Spotify Connect and favorite write controls until supported.
MilkDrop, spectrum, oscilloscope, EQ, mono and channel balance are not supported
because this engine does not expose PCM. Their settings and mini-player actions
are unavailable in Apple mode. Skin artwork can still contain a fixed EQ button;
it does not open an unsupported effect. Bitrate/sample-rate labels stay blank
instead of reporting the legacy decoder's values.
Non-Windows builds report unsupported Apple playback.

## Motion and Now Playing

PR #13's motion is integrated on `feat/apple-release-polish`. It adds cover
hover/lift, dancing playing indicators, seek hover, cover crossfade, heart pop,
queue arrivals, art-colored headers, page fades and **Reduce motion**. Apple
Now Playing shows artwork, transport controls and LRCLIB lyrics when available.
Open it with **L** or **Ctrl+Shift+K**; **Esc** returns. **Ctrl+M** opens
the mini-player.

### Apple lyrics

In [PR #19](https://github.com/jackkayser2005/applifast/pull/19), on
`feat/apple-lyrics`, the player-bar Lyrics button opens the existing side panel.
Full-screen Now Playing shows a large cover beside lyrics at normal widths and
stacks a small cover, controls and lyrics in narrow windows. Timed lines seek
through the existing Apple player commands. Follow, manual scrolling, smooth
line highlighting and Reduce motion reuse the existing controls. Missing lyrics,
instrumental tracks, loading and failures have explicit states; failed requests
offer Try again rather than retrying automatically every frame.

Apple tracks use the existing [LRCLIB](https://lrclib.net/docs) exact/search
lookup by artist, title, album and duration. These are community lyrics, not
Apple's transcription; availability and timing can differ by recording. Apple
identifiers never enter the Spotify lyrics request path. Opening the side panel
or Now Playing starts a lookup; keeping it open follows subsequent songs. Only
song metadata goes to `https://lrclib.net/api/get` and `/api/search`, using the
existing HTTP client. No Apple tokens, browser profile or audio are sent.

The existing lyrics JSON cache under the separate Applifast cache directory
stores public metadata-keyed answers for 30 days, including no-match results.
It is independent of account authorization and can be reused after sign-out.
Sign-out, authorization cancellation, token import and host replacement cancel
pending lyric work and clear shown lyrics. New lookups cancel the preceding
one. URI and request generations reject late results, including an older answer
for the same song after reauthorization. No dependency or settings format change
is introduced. Real-account lyric matching, timing and playback acceptance remain
pending separately from deterministic fixtures.

Windows default/demo contribution checks pass: 982 and 1,011 library tests
respectively (2 ignored in each suite), their binary/integration targets,
strict all-target Clippy, formatting and default Rustdoc/doc tests. Focused
regressions cover same-song stale answers after sign-out/retry, backend
cancellation, Apple URI exclusion from Spotify requests, timed-line seeking
and wide/narrow layout. The existing line-following and Reduce motion tests
also pass. The isolated native credential-store dummy round trip and Node
bridge/token self-checks pass. Optional all-feature projectM/vcpkg, site/Nix,
non-Windows and real-account lyric acceptance remain pending.

A public LRCLIB metadata-only smoke request returned HTTP 200 with plain and
synced lyric fields. It logged only response status and field-presence booleans,
not lyric text or credentials. This checks service reachability, not account
matching or the app's playback timing. The initial token self-check command
used a nonexistent filename; the existing `generate-token.cjs --self-test`
passed after correcting the command.

The follow-up read-only account check can be run with:

```powershell
cargo test --locked --lib app::apple::tests::native_host_reads_home_and_album_dates -- --ignored --exact --nocapture
```

It starts no playback or library writes. It checks the four Home responses and
the album-date mapping through the native host and application response path,
and logs aggregate counts only. This machine's run failed during host
initialization, before authorization readiness or any Apple API read. Fixed
redacted setup diagnoses did not identify the cause. Raw host-error logging was
rejected by automatic approval review because it could expose authorization or
SDK data; that logging was removed. That host run supplies no real-account
Home/date result. The separate read-only account diagnostic above later passed;
it does not satisfy native-host acceptance.

After adding this opt-in check, strict default/demo all-target Clippy and
formatting pass. Both full all-target suites pass again (982/1,011 library tests,
now 3 ignored in each). The first demo rerun hit the existing settings-save
branding test; its isolated run and the full demo rerun passed without changing
or weakening the test. The packaged preview predates only this test/docs
follow-up; its application code is unchanged.

The [lyrics comparison](review-lyrics/index.html) records matching Windows
light/dark and narrow/normal before/after captures and full-screen failure,
loading, instrumental and no-match states. The baseline side panel is forced
open with a demo flag and sample text; that baseline has no enabled Apple
lyrics lookup or player-bar entry. Stills do not prove animation or audio.
All 32 images were inspected, and the gallery selectors and PNG dimensions
were checked. The existing narrow side-panel layout remains cramped in both
the baseline and candidate; this slice improves full-screen lyric layout.

The local combined preview is `dist/applifast-windows-preview-2c6c576.zip`
(23,031,492 bytes), built from `2c6c576b3ad77bcf45782233386a857c5a39d790`.
ZIP SHA256: `699571e0fb473ec13c6cc589c9c40998aeb57f8c73f6e641cd05eb59effa7acf`.
Executable SHA256: `5df1d68253691479b4ff5fae4b00252df42c4939eaa677e7fcff382cfa367781`.
The archive contains only Applifast.exe, LICENSE, README.txt and BUILD.txt.
It includes the Home/album-date fixes and these lyrics, but the compact sidebar
Favorites shelf and public token onboarding remain unfinished. This is a
debug/demo Windows x64 development preview with static CRT and inherited
version 0.12.0, not a public release.

**Ambient Pulse** is an optional decorative animation shared by both players.
Enable it in Appearance, click empty player-bar space, or use the mini-player's
V menu. It follows the playhead and album colors, freezes while paused, changes
phase on a seek, and works at zero volume. It does not analyze the audio.
Reduce motion holds it still. Visible playing views request at most 30 pulse
frames per second; off-screen and minimized views request none from the pulse.
The default is off. This adds `ambient_pulse` to the existing settings JSON;
older settings remain readable. No new files, dependencies or network endpoints
are introduced by this animation.

Apple Account settings provide developer-token renewal and sign-out. Unsupported
legacy decoder, Spotify account, proxy, audio-cache and upstream-update settings
are hidden. Artwork already uses a disk cache. Loaded Apple song metadata and
the full local queue now persist across restarts; audio remains streamed by MusicKit.

## Apple Home and recently added albums

Home uses the documented Apple Music feeds for Recently Played, Recently Added,
Heavy Rotation and Recommendations. It replaces the inherited Spotify top-artist,
top-song and discovery requests. Albums and playlists open their Apple detail
pages; song cards preserve their original catalog or library identity. Artist
cards navigate without pretending to play an unsupported artist context. Stations
and other unsupported resource types are omitted. Empty feeds explain that no
items were returned; errors offer Retry instead of remaining on Loading.

A Home visit reads the four feeds once per signed-in session. Refresh replaces
those reads, retains shown cards, and rejects late responses. Load more requests
one additional page per unfinished feed, retaining at most 64 distinct cards per
feed. Recommendations use Apple's included contents (a preview, not every nested
relationship page). These metadata reads use the existing isolated MusicKit host
and no extra credentials, dependencies or app-operated service. Home metadata
stays in memory and is cleared on sign-out.

The library's Recently added album sort retains Apple's optional `dateAdded`.
Full timestamps and date-only values are accepted; a year-only date sorts at the
start of that year. Missing or invalid dates remain unknown and sort last, rather
than substituting the album's release date. Real-account feed and sorting checks
remain pending; automated fixtures do not establish that acceptance.

Apple's documented library song attributes do not include a song add date. The
Songs table omits the inherited empty Date added column in Apple mode. Album
add dates are not silently assigned to individual songs. Use the Home Recently
Added feed or the library album sort for the supported chronological views.

Sources: [recently played resources](https://developer.apple.com/documentation/applemusicapi/get-recently-played-resources),
[recently added resources](https://developer.apple.com/documentation/applemusicapi/get-recently-added-resources),
[heavy rotation](https://developer.apple.com/documentation/applemusicapi/get-heavy-rotation-content),
[default recommendations](https://developer.apple.com/documentation/applemusicapi/get-all-recommendations),
and [library album added date](https://developer.apple.com/documentation/applemusicapi/libraryalbums/attributes-data.dictionary).

### Home validation on Windows

The default all-target suite passes (981 library tests, 2 ignored), as does demo
(1,009 library tests, 2 ignored). Strict default/demo all-target Clippy,
formatting, gettext checks, default Rustdoc/doc tests, Node bridge/token
self-checks and 11 host tests pass. Both isolated Windows Credential Manager
dummy-grant round trips pass. An unrelated temporary-filesystem credential test
failed in the first full attempt; its isolated run and the complete rerun passed.
No lint or test was relaxed. Optional all-feature projectM checks remain blocked
by the missing vcpkg installation; Ruby/Bundler/Jekyll/Nix and non-Windows
runtime/compilation coverage remain unavailable here. Real-account Home results
and album sorting are not claimed from these fixtures.

The [Home review](review-home/index.html) compares Windows light/dark and
narrow/normal frames and records the intentional Home feed-data change separately
from matching Songs rows. It also shows loading, empty and error states. The user
requested these Home/date fixes and chose the left sidebar for the upcoming
Favorites shelf. The Home preview does not include lyrics or that shelf; lyrics
are a separate following feature slice.

The focused draft is [PR #18](https://github.com/jackkayser2005/applifast/pull/18),
stacked on #17. The local development ZIP is
`dist/applifast-windows-preview-1996c0e.zip` (23,036,311 bytes), SHA256
`786440d16f7295e2ff9378da1bd3162674fe707d7c20034011b6624a18c4d326`.
Its executable SHA256 is
`63b0f4db6702d0ca899a581ae0a0adc9d3dc7d0a0348b033c37aa64c696760bf`.
The ZIP contains only the executable, MIT license and usage/build notes. It is
a debug/demo Windows x64 build with static CRT and inherited version 0.12.0,
not a public release. Earlier preview packages do not contain the Home fixes.

## Reading favorites

On `feat/apple-favorites-view`, **Show only favorites** in Songs filters the
loaded library using Apple's optional `inFavorites` boolean. The heading becomes
Favorites and its count covers only these loaded rows. Library membership,
ratings, a catalog match and a missing flag never imply a favorite. Unknown and
false flags remain distinct in metadata. Unavailable favorited songs stay visible.
The filtered header and song rows play only the displayed playable songs, even
with shuffle enabled. Sorting and text filtering still apply within the subset.

An empty loaded subset explains its scope and offers **Load more songs** when
Apple supplies another page. It does not claim the entire account has no favorites
or download every page automatically. Clearing the checkbox restores all loaded
songs. Sign-out clears this in-memory view preference along with account data.
Favorite metadata persists as an optional field in the existing `apple-session.json`;
older snapshots load with unknown favorite state. No new storage, dependencies,
credentials or network endpoints are added. Refresh Songs to pick up changes made
in Apple Music.

This is a read/filter slice. Player/mini-player/row favorite write controls and
bulk writes remain pending. The navigation page and shelf are implemented below.
Apple's [favorite state](https://developer.apple.com/documentation/applemusicapi/librarysongs/attributes-data.dictionary)
is separate from ratings. Its [add endpoint](https://developer.apple.com/documentation/applemusicapi/add-resource-to-favorites)
returns 202 with no body and may ignore IDs, so an acknowledgement alone cannot
confirm a favorite. A supported removal route has not been established in the
official API documentation checked for this slice; no undocumented DELETE or
ratings substitution has been implemented.

Deterministic sample states: `--demo-show favorites` and `favorites-empty`.
They test filtering and rendering, not an account's favorite metadata or playback.
Real-account favorite reads and restart restoration remain runtime acceptance gates.

### Requesting favorite metadata

The favorite-metadata follow-up explicitly requests `extend=inFavorites` on
library song reads. Apple's [library song endpoint](https://developer.apple.com/documentation/applemusicapi/get-all-library-songs)
supports attribute extensions, and its [song attributes](https://developer.apple.com/documentation/applemusicapi/librarysongs/attributes-data.dictionary)
include the optional favorite boolean. The MusicKit bridge adds that extension
to initial pages, continuation pages and direct library song reads. It keeps
existing query parameters and requests album/artist relationships when a next
URL omits them. Original library IDs and playback parameters remain unchanged.
Catalog/search/playlist routes are not extended. The native read boundary permits
only this named extension on library song collection/detail paths; other extension
names, catalog paths and playlist-track paths remain rejected.

This changes reads, not favorite writes or the interface. Existing optional flags
and the atomic restart cache retain their format; missing/string flags stay unknown,
and ratings do not imply favorites. After updating, use **Songs > Refresh** to
replace the cached library span with newly requested metadata. No extra endpoint,
dependency, credential or cache is introduced. The read-only account diagnostic
above verified explicit flags on 100 sampled songs, including 30 favorites.
Complete-library favorite coverage and the visible shelf still require verification.
No competing account host has been started while the older preview owns its profile.

The bridge self-check verifies first/next/detail requests, preserved offset and
album/artist inclusion, a single favorite extension, untouched catalog search,
optional boolean flags and original uploaded IDs. All 11 standalone host tests
pass, plus strict standalone Clippy/formatting and its isolated Windows Credential
Manager dummy round trip. This read-only change retains the existing Favorites
[visual comparison](review-favorites-shelf/index.html); no layout or control changed.
Integrated Windows checks also pass: 985 default and 1,016 demo library tests
(three account checks ignored in each), binary/integration suites, strict
default/demo Clippy, formatting, gettext, default doc tests, strict Rustdoc and
token-generator self-check. Optional projectM/all-features, launcher/site/Nix,
non-Windows and real-account acceptance remain separate pending gates.
Draft [PR #22](https://github.com/jackkayser2005/applifast/pull/22) is stacked on
the search slice. Combined local preview: `dist/applifast-preview-9d4c906/Applifast.exe`.
The four-file portable archive is `dist/applifast-windows-preview-9d4c906.zip`
(23,057,452 bytes), SHA256
`72333a6a29a54bd9b461a37c0a673a53000e8afd560da7ef52c322028b02bbcf`.
Executable SHA256: `0bdb8081b6b9e090b9cf9230bf63bdfae4c6e0c12aad5d8ea54d671abb06efcf`.
This is a Windows x64 debug/static-CRT/demo preview with inherited version `0.12.0`,
not an installer or public release. Quit older previews through the tray before
starting it. Songs Refresh picks up the metadata request changes; Home Refresh
retries Recently added and the other feeds. No credential is in the package.

### Sidebar Favorites shelf

On `feat/apple-favorites-shelf`, Apple navigation has a separate Favorites page.
It reuses the Songs table, sorting, text filtering, selection, queue/menu actions,
Refresh and Load more, with separate page identity, row caches and filter state.
Opening it does not change the Songs checkbox. Back/Forward and the existing
last-page session value preserve this page. Only `inFavorites: true` appears;
unknown and false states are not guessed. Unavailable favorites remain visible.

The sidebar shows the first loaded favorites in library order. A click starts
the playable favorite context at that song; unavailable clicks report the
existing actionable error without replacing playback. Titles truncate through
the existing bidi helper, with full title/artist on hover and keyboard focus.
The shelf scrolls when space is tight so the library remains reachable.
**Show all favorites** opens the dedicated page, including its loaded-subset
explanation and pagination. This does not claim every account favorite is loaded.

Appearance adds **Favorites in sidebar**, from zero to ten entries, default five.
Zero hides the compact shelf while retaining Favorites navigation. The additive
`favorite_shelf_count` preference is serialized through the existing atomic
settings file; older settings default to five, and loading/changes cap it at ten.
No new cache, credentials, dependency or network endpoint is introduced.
Sign-out uses the existing account clearing, so these private song rows disappear.
Favorite metadata already persists in the existing restart cache; its real-account
read/restoration acceptance remains pending separately from fixtures.

Demo flags: `favorites-page` opens the separate page; `favorites-shelf-hidden`
hides the shelf. Combine `favorites-empty,favorites-page` for the no-known-flags
state. Deterministic tests exercise shelf clicks, Show all, independent navigation,
playable context identity, unavailable rows, count bounds and older preferences.
Windows checks pass: 983 default library tests and 1,013 demo library tests,
with three ignored account checks in each, plus the binary/integration suites.
Strict default/demo all-target Clippy, formatting, gettext checks, default doc
tests and Rustdoc pass. The full suite caught a changed Songs filter-cache key;
restoring its original key and using a separate Favorites key fixed the shared
cause, and both complete suites passed afterward. No test or lint was relaxed.
The [Favorites sidebar review](review-favorites-shelf/index.html) includes 32
inspected native Windows light/dark and narrow/normal frames for Songs, Favorites,
Appearance, empty favorites and hidden shelf. Selector paths and PNG dimensions
are checked. Real-account favorite completeness and playback remain pending.
Bridge and token-generator self-checks also pass. The launcher-install check
cannot complete here because its Ruby YAML parser and Unix `true` utility are
unavailable. The existing projectM/vcpkg, site/Nix and non-Windows coverage
limitations apply.

The Home follow-up found the user was running preview `3a87e21`, which predates
the Apple Home integration in `1996c0e` and the combined lyrics preview `2c6c576`.
The read-only account check stopped before any feed request with validated
WebView2 HRESULT `0x8007139F`; the older preview still owned the app's browser
profile. This does not establish a failed Recently Added endpoint or a repaired
real-account feed. Quit older previews through the tray before testing the new
one. Diagnostics print only fixed classifications and validated numeric HRESULTs,
never authorization or SDK error text.

The combined [draft PR #20](https://github.com/jackkayser2005/applifast/pull/20)
preview is built from `36de3583e3408ae404fbd5cedd4c95e6675ebb2e`. Run
`dist/applifast-preview-36de358/Applifast.exe` after quitting older tray instances.
It includes Home, lyrics and the sidebar shelf. The executable is 63,895,552
bytes, SHA-256 `1353df7076ccbdb1ed288e96d0a646eac665442dfe4bc4ee1b81e0eb2c9e3fe8`.
`dist/applifast-windows-preview-36de358.zip` is 23,042,506 bytes, SHA-256
`c0b1cefe940013d43b58f0023f05ddeef16b962691f267a80377c199e8b05a50`.
Its verified whitelist is the executable, LICENSE, README.txt and BUILD.txt.
This remains a Windows x64 MSVC debug/demo preview with a static CRT and inherited
`spotifast 0.12.0` version, not a public release or installer. It carries no keys
or tokens; existing local authorization is reused. Runtime gates above remain
pending and the package is ignored by Git.

Windows validation for the preceding favorites read/filter slice: 979 default
and 1,006 demo library tests,
all default/demo binary and integration targets, strict default/demo Clippy,
ten playback-boundary tests, both isolated native credential-store round trips,
Node bridge/token checks, formatting, generated catalogs, default doctests and
Rustdoc with warnings denied pass. All-feature checks remain blocked by the
unchanged optional projectM vcpkg prerequisite. Jekyll/Nix and non-Windows builds
have not been verified here. The ignored real-account playback/restoration test
was deferred because an older app instance was still running.
The [favorites comparison](review-favorites/index.html) has four matching
light/dark, narrow/normal Windows pairs and six selected/empty-state captures.
All fourteen captures were inspected; their dimensions and gallery selectors were
checked. At the narrow size, the existing hero style ellipsizes its description.
The [comparison index](reviews.html) links all integration reviews.

## Apple search pagination

Apple search follows the per-resource `next` URLs returned by the existing
[library search](https://developer.apple.com/documentation/applemusicapi/search-for-library-resources)
and catalog search requests. Library and catalog cursors are held separately for
songs, albums, artists and playlists. **Load more** loads the selected type, or
every available type for All, without another click while those requests wait.
It appends unique resource URIs in response order. The same song appearing as a
library resource and a catalog resource stays distinct; an uploaded song is never
replaced with its catalog match. Queue occurrence rules are unchanged.

Cursor validation requires the same search path, storefront, committed query,
resource type and one nonempty offset. External URLs, other queries/types, repeated
cursors and pages making no progress stop pagination. Changing or clearing a query
removes pending search IDs and cursors; late replies cannot refill the new view.
Sign-out retains the existing generation cancellation and account clearing.
Failures keep successful results and the failed cursor available for retry.
Queries that cannot pass request validation show an error rather than loading
forever. No dependency, credential, setting, cache format or network destination
is added. Search remains on the existing MusicKit host and Apple API endpoints.

Apple filters omit Podcasts and Episodes, and an empty search names Apple Music.
An Apple track's top-result Play action uses its original track URI. Library
artist cards open their library albums without offering catalog-only top-song
playback. Catalog artists retain that action. The existing card layout is retained.
Demo flags `apple-search-pages`, `apple-search-loading`, `apple-search-error` and
`apple-search-empty` cover these states without authorization or private metadata.

Windows validation: 985 default and 1,016 demo library tests pass, with three
ignored account checks in each, plus the binary/integration suites. Regressions
cover separate library/catalog identities, uploaded-song preservation, cursor
validation, retry after a partial failure, selected-filter pagination, duplicate
requests/replies, stale search answers, sign-out and the actual UI Play/Load more
actions. Strict default/demo all-target Clippy, formatting, gettext, default doc
tests, strict Rustdoc, bridge and token-generator self-checks pass.
The [search comparison](review-search/index.html) contains 24 inspected native
Windows light/dark and narrow/normal frames. Selector paths and PNG dimensions
also pass; this is not a browser-rendering or real-account acceptance claim.
Draft [PR #21](https://github.com/jackkayser2005/applifast/pull/21) is stacked on
the Favorites sidebar slice. Its combined local Windows debug preview is
`dist/applifast-preview-3866237/Applifast.exe`, with portable archive
`dist/applifast-windows-preview-3866237.zip` (23,057,219 bytes). The ZIP contains
only the executable, LICENSE, README.txt and BUILD.txt. ZIP SHA256:
`b3c17497945091c455c0be61b573e030c9c6cb28bd475dada7313c4cfc42699f`.
Executable SHA256: `19f6301141539ad611ba8edbb7e001486502920b40089a4ddba32e248c7cdffd`.
It includes Home, lyrics, Favorites and this search slice. Quit older builds
through the tray before starting it, then use Home Refresh to retry the feeds.
This debug/static-CRT/demo package keeps inherited version `0.12.0` and is not an
installer or public release. The whitelist excludes all keys, tokens and caches.
Real-account search pagination remains pending while the older preview owns the
profile. Optional all-features checks retain the recorded projectM/vcpkg blocker;
Ruby/Unix launcher tools, Jekyll/Nix and non-Windows coverage remain unavailable.

## Restart restoration

`apple-session.json` in Applifast's state directory stores only metadata and the
local occurrence queue, not audio or credentials. It is atomically replaced on
the backend, with a 32 MiB ceiling. Playback checkpoints are saved approximately
every 15 seconds and when either window closes or the app quits. Queue changes
and library pages use the existing two-second session-save debounce.

Restoration waits for the playback host's saved authorization. A SHA-256 tag of
that grant, computed in the native host, and the storefront must match before
rows are shown. A new authorization token can intentionally invalidate the old
snapshot. Sign-out invalidates late cache responses and removes the snapshot;
cache commands execute in order so a pending write cannot recreate it afterward.
Tokens remain in Windows Credential Manager. The host uses the existing SHA-256
dependency for this tag, with no additional crate versions or network endpoints.

Cached songs remain visible while their loaded span refreshes. Collection detail
pages still load from Apple when opened. Unavailable uploads and original playback
parameters remain intact; restoration never substitutes a catalog match. A failed
seek or item load retains the queue and reports a playback error. Restart
restoration still needs a real-account Windows runtime check.

For a deterministic preview without changing your account:

```powershell
cargo run --locked --features demo -- --demo --demo-page liked --demo-show queue,ambient-pulse
```

The combined Windows update passes strict default/demo Clippy, 972 default and
996 demo library tests, the default/demo binary and integration suites, eight playback-boundary tests, the isolated
native credential-store round trip, Node bridge/token checks and generated-catalog
verification. Default Rustdoc with warnings denied and default doctests also pass.
The older results below describe the preceding integration build.
The [candidate gallery](review-polish/index.html) covers light/dark and narrow/normal
Windows views. It is not a matching motion comparison: a baseline build is pending.
Additional 760-point queue captures deliberately bypass the native panel minimum
and show clipping at that forced size. Ordinary window resizing raises its minimum
while the panels are open; real resize and window-manager coverage are pending.
Matching motion comparisons and real app restart acceptance remain separate checks.
The real-account restoration check was deferred while
the older app was playing, to avoid interrupting it.

`packaging/applifast-preview.txt` accompanies the local Windows development ZIP.
It describes saved authorization, restart testing and the credential-free demo.
The debug preview retains upstream package version 0.12.0 and is not a release.
The Windows packaging-launcher attempt fails without the Unix `true` executable
and Ruby YAML tooling; that contribution check has not passed here.

## Playlist creation and queue saving

On `feat/apple-playlists`, the library's **+** dialog creates an Apple library
playlist. The queue's **Save as a playlist** button saves the current song followed
by every upcoming occurrence in play order, including repeated songs. Selected
songs can also use **New playlist** in the existing playlist picker. The Public
choice uses Apple's documented `isPublic` attribute.

Creation posts once to `/v1/me/library/playlists`, with an optional tracks
relationship. Catalog resources use `songs` and their original resource ID;
uploaded/library resources use `library-songs` and their original library ID.
Playback parameters and catalog matches never replace those identities. The
existing 1,000-occurrence boundary also applies to one playlist request. Unknown
selected songs stop the whole request and ask for a reload; none are silently lost.

The new row and submitted songs appear immediately. After Apple's acknowledgement,
the app reads the library and every track page after a two-second delay. Lagging
answers cannot erase the submitted occurrences. A catalog song may legitimately
return as a library song carrying the same catalog ID; an upload must retain its
library ID. Confirmation stops after three attempts and offers Refresh while
keeping the songs visible. It never automatically repeats a write. An ambiguous
failure asks you to inspect the library before retrying, since Apple may already
have accepted it. Sign-out invalidates late replies, aborts pending write fetches,
clears write state and prevents queued writes from running under another account.

Writes serialize independently of playback controls on the existing host. This
adds no files, dependencies, credential storage or telemetry. Only transient
pending-write/confirmation state is held in memory; playlist details are read
from Apple again after restart. This creation slice validates the documented
append endpoint; the following slice exposes it in the interface.
Rename, deletion, reorder, covers and collaborative controls remain unsupported.

The deterministic demo accepts `--demo-show create`, `playlist-saving`,
`playlist-created` or `playlist-error` (alongside theme flags). These use sample
resources and establish UI behavior only. Boundary and app regressions cover
duplicate occurrences, original IDs, pagination, stale reads, rollback, bounded
confirmation, cancellation and sign-out. Real-account creation/public state,
uploaded members and server consistency still require Windows runtime acceptance.

API evidence: [create a playlist](https://developer.apple.com/documentation/applemusicapi/create-a-new-library-playlist),
[creation request](https://developer.apple.com/documentation/applemusicapi/libraryplaylistcreationrequest),
[allowed track types](https://developer.apple.com/documentation/applemusicapi/libraryplaylisttracksrequest/data-data.dictionary).
MusicKit JS v3's current APISession accepts `fetchOptions` in the third argument
to `music.api.music`; the checked implementation uses POST JSON there.
The SDK script fetched on October 8, 2026 from Apple's `js-cdn.music.apple.com`
has SHA-256 `21908b72bdcdeea3ec91fa40b105d0b9a2ba07c2a3fe32ea77edcd26e735012c`.
No SDK source is copied into this repository; the existing host loads Apple's CDN.

Windows validation for this slice: 974 default and 999 demo library tests pass,
along with all default/demo binary and integration targets, strict default/demo
Clippy, nine host boundary tests, the isolated native credential-store round trip,
Node bridge/token checks, formatting and generated-catalog verification. Default
doctests and Rustdoc with warnings denied pass. All-feature Clippy, tests,
doctests and Rustdoc stop at the optional projectM build because
`VCPKG_INSTALLATION_ROOT` is unset. Linux/macOS builds,
real-account playlist writes and release acceptance are not claimed.
The [playlist comparison](review-playlists/index.html) has matching light/dark,
narrow/normal Windows captures plus saving, success and retry states. The common
dialog captures match pixel-for-pixel; Apple playlist attribution no longer falls
back to Spotify. These captures use sample resources rather than your account.

## Appending songs, albums and the queue

On `feat/apple-playlist-additions`, the existing **Add to playlist** picker offers
only library playlists whose Apple `canEdit` attribute is true. Missing/false
permission and catalog playlists are excluded. This grants appending only;
rename, delete, remove, reorder and collaborative editing stay unavailable.
The sidebar accepts selected-song drops on those writable playlists.
An album's menu offers the same picker, including **New playlist**. Every album
track page loads before any write or creation draft; a failed page adds nothing.
Right-click the queue's **Save as a playlist** button to append the current song
and every upcoming occurrence to an existing playlist. Left-click still creates
a new playlist. Queue order and repeated songs are preserved.

The destination's complete track list loads before checking duplicates. Songs
already present use the existing **Add anyway** confirmation. Cancelling that
dialog sends no write; sign-out cancels pending loads. Once a write starts, the added rows and
count appear immediately. Original resource IDs go to Apple's
`POST /v1/me/library/playlists/{id}/tracks`; catalog/playback IDs never replace
uploaded/library identities. The existing 1,000-song request bound applies and
an unknown selected song rejects the entire batch.

On a successful response, the existing two-second, three-attempt paginated
confirmation protects every expected occurrence against lagging reads. Failed
writes roll back only the new suffix, ask you to inspect the playlist before
retrying and never repeat a write automatically. Old reads, empty/repeating
continuations and sign-out cannot confirm or silently drain an addition. Pending
destination/album pages stay in memory until their operation finishes. Loading
very large destinations currently holds their full metadata list for exact
duplicate and occurrence checks; this still needs integrated memory measurement.
No new dependencies, disk files, credentials or telemetry are introduced.

API evidence: [append tracks](https://developer.apple.com/documentation/applemusicapi/add-tracks-to-a-library-playlist)
and [library playlist permissions](https://developer.apple.com/documentation/applemusicapi/libraryplaylists/attributes-data.dictionary).
Apple documents a successful append as HTTP 204 with no response body and notes
that new resources can take time to appear. The existing host handles that empty
response independently of playback commands.

The demo exposes `playlist-appending`, `playlist-appended`, `playlist-append-error`
and `playlist-duplicates`. These are sample responses, not real-account acceptance.
Windows checks pass: 978 default and 1,004 demo library tests, all default/demo
binary and integration targets, strict default/demo Clippy, formatting and Node
bridge/token checks. Seven focused Apple app tests cover both creation and
appending; keyboard picker and deterministic pending/confirmed/error/duplicate
states also pass. The [comparison](review-playlist-additions/index.html) records
matching light/dark, narrow/normal Windows frames and append states; open
album/queue menu captures remain pending. Three common pairs are byte-identical;
the dark/narrow pair differs only by one channel level at 80 toast-icon pixels.
Existing all-feature projectM/vcpkg,
packaging/site/Nix and hosted CI gates remain outstanding. Windows real-account writes, permission rejection,
network interruption and restart acceptance remain unverified.

## Storage and network

Settings/state/cache paths use `applifast`, independent of Spotifast. The
single-instance and media-control names also use Applifast. The development
binary name and old packaging have not yet been renamed for distribution.
Developer and user tokens remain in Windows Credential Manager under
`local.applifast.playback-probe`, reusing the account already authorized locally.
The separate browser profile and embedded playback assets live under
`%LOCALAPPDATA%/Applifast/playback-probe`; the profile is not the user's browser
profile. The app never needs the `.p8` file at runtime.

MusicKit loads from Apple's v3 CDN and talks to Apple's authentication, library,
catalog/search, media and license services. Reads use validated relative paths
for known library/catalog collections and artist views. They run independently
of playback commands and never expose credential fields. Legacy Spotify API,
auth and playback answers cannot overwrite Apple state. There is no hosted Applifast backend, app telemetry,
or decrypted-media persistence. SDK/runtime behavior remains governed by Apple
and Microsoft. Upstream update checks are disabled; the update configuration
points to this fork, so it cannot select an upstream Spotifast release.

## Validation and pending acceptance

The standalone [evidence report](playback-evidence.md) records full catalog
playback, hidden mixed queues, seeking and measured resource use. Its original
250 MiB memory gate failed: 285.406 MiB playing and 278.297 MiB paused. The user
explicitly accepted this footprint for initial integration. That is a changed
product constraint, not a passing result against the original budget.

The integrated-host ignored test uses saved local authorization, loads the real
library, starts muted playback, checks the SDK playhead after seeking, then
pauses and shuts down. It never logs tokens or library metadata:

```powershell
cargo test --locked --lib apple::tests::native_host_restores_library_plays_seeks_and_pauses -- --ignored --exact --nocapture
```

Ordinary unit/demo tests use deterministic data and do not restore account
credentials. Bridge regressions run with
`node prototypes/apple-playback/web/test-bridge.cjs`. For matching UI captures,
`--features demo -- --demo-shot PATH --demo-size 1100x760 --demo-page liked`
uses the same song metadata as the retained legacy view; add `--demo-show legacy`
for Before and `--demo-show light` for light theme.

Windows validation on 2026-10-07 passed: strict default/demo Clippy, 957 library
tests plus the binary/integration suites, both isolated native credential-store
round trips, seven host protocol/origin tests, Node token/bridge checks, and the
ignored real-account integrated-host test with 100 library rows. That host test
checked real advancing playback, the SDK's actual playhead after seeking, and
Pause issued during seeking. Real library shelves and catalog/library searches
also returned successfully. Available library album, playlist and artist details,
their track/album relationships, and a catalog artist's top-songs view passed
the expanded account check. Final default/demo verification passed with one
compiler job after Windows exhausted memory during parallel linking.
A pending SDK seek promise cannot block later
controls; a late seek completion must preserve the requested pause.
The profile's disk cache is cleared at host startup so embedded bridge updates
are applied without clearing saved authorization.

[Visual comparison](review/index.html) includes matching light/dark and
760/1100-point-wide Windows captures, plus signed-out, connecting, loading and
error states. The baseline is the retained legacy renderer at this branch's
revision, rather than a separate build of an upstream release.

Integrated aggregate resource measurement, a one-hour gaming session, complete
track listening in the new UI, real sign-out/cancellation/expiry, output-device
changes, and tray/window recreation remain runtime acceptance checks. The
standalone CPU measurements do not establish the integrated footprint.
Windows tests do not establish Linux/macOS compilation or runtime coverage.
All-feature checks require the optional projectM/vcpkg toolchain; Nix's vendor
hash must be refreshed after the changed lockfile and its build verified in CI.
The attempted all-feature Clippy check failed because
`VCPKG_INSTALLATION_ROOT` is unset. Packaging/site checks are blocked by missing
Ruby/Bundler and Unix shell tooling; Nix is absent locally. These checks are
pending, not passing.
The draft PR has no GitHub checks: Actions permissions report enabled, but the
fork's workflow inventory is empty and activating `ci.yml` returns HTTP 404
despite that file existing on `main`. CI has not run. This repository workflow
registration must be resolved before relying on hosted checks or refreshing the
Nix vendor hash from its build result.
