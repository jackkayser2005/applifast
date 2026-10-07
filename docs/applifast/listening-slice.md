# First Windows listening slice

Applifast starts a Rust/egui Songs view and an independent MusicKit JS v3 host.
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

- Load synced library songs in validated 100-song pages and filter loaded rows.
- Double-click a playable song to start the loaded-song context; play/pause,
  seek, volume, next and previous use the same action path as desktop media keys.
- Preserve original library IDs and Apple's playback parameters, separately
  from catalog IDs. Duplicate occurrences remain distinct in the playback queue.
- Keep unavailable songs visible. A playback failure retains the queue and
  shows an error rather than silently skipping the song.
- Hold optimistic song, pause and seek state against older playback events.
  Sign-out invalidates pending account responses, clears in-memory account data,
  deletes the saved user token, and requests browser-profile clearing.

The first context contains at most 1,000 playable loaded songs, the host's
validated queue limit. Catalog browsing/search, albums, playlists, artist pages,
manual queue additions, shuffle, queue/session restoration, and the mini player
are follow-up integration slices. The engine accepts catalog descriptors, but
this Songs view only presents the synced library. Cloud-only upload playback
remains unverified; no upload is silently replaced with a catalog match.
MilkDrop, spectrum and EQ are absent because this engine does not expose PCM.
Non-Windows builds report unsupported Apple playback.

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
media and license services. There is no hosted Applifast backend, app telemetry,
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

Windows validation on 2026-10-07 passed: strict default/demo Clippy, 955 library
tests plus the binary/integration suites, both isolated native credential-store
round trips, six host protocol/origin tests, Node token/bridge checks, and the
ignored real-account integrated-host test with 100 library rows. That host test
checked real advancing playback, the SDK's actual playhead after seeking, and
Pause issued during seeking. A pending SDK seek promise cannot block later
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
