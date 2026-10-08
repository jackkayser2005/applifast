# First Windows listening slice

Applifast reuses the original Rust/egui interface and an independent MusicKit JS v3 host.
The host uses the installed Evergreen WebView2 runtime on its own COM STA thread
with a message pump. Commands and sanitized events cross channels through the
existing asynchronous backend. Closing an egui window does not own or destroy
the playback host. Apple authorization is the only visible browser popup.

## Local setup

Follow the [developer-token setup](../../prototypes/apple-playback/README.md).
Keep the `.p8` key and generated JWT in ignored `.secrets/apple-music/`, never
in a commit or chat. The app imports the JWT file, not the signing key. On
Windows, run `cargo run --locked`. The executable is still named `spotifast.exe`
during this development slice; its window and local identity are Applifast.

If no saved grant exists, enter the absolute path to `developer-token.txt`, click
**Import token**, then **Sign in with Apple**. Enter the subscriber's credentials
only in Apple's popup. Existing grants from the playback probe restore without
another import. An expired developer JWT requires generating and importing a
new one. Authorization errors retain an explicit retry action.

## Supported now

- Reuse the original sidebar, tables, artwork, search, account menu and player bar.
  Apple authorization occupies the existing sign-in card. No replacement app shell
  or palette redesign is included. The synced song shelf is called **Songs**.
- Load synced library songs in validated 100-song pages and filter loaded rows.
- Browse library albums, artists and playlists, including collection detail pages
  and paginated tracks. Catalog and library search share the existing search view;
  library and catalog resources retain distinct identities. Catalog artist pages
  request Apple's top-songs view; library artists show their library albums.
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
It restores paused. Home/discovery/recommendations,
favorites and playlist writes remain follow-up integration work.
The existing mini player uses the same playback actions, but its window lifecycle
still needs real runtime acceptance. Cloud-only upload playback
remains unverified; no upload is silently replaced with a catalog match.
The player bar omits Spotify Connect, favorites and lyrics controls until supported.
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
Now Playing shows artwork and transport controls without requesting legacy
lyrics. Open it with **L** or **Ctrl+Shift+K**; **Esc** returns. **Ctrl+M** opens
the mini-player.

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
