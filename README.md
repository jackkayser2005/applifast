# Applifast

Apple Music for Windows, with a native Rust/egui interface and MusicKit JS
playback in an independent WebView2 host. Built for listening from the desktop,
tray or mini player while other apps are running.

**Development preview, Windows x64.** This fork is not yet a public beta or a
feature-parity release. Linux and macOS playback are unsupported. The original
Spotifast downloads install a different product.

See the [Windows beta checklist](docs/applifast/release-readiness.md) for the
next private trial, current evidence and remaining public-release gates.

## Try the preview

Invited testers: get the preview installer or portable ZIP and its checksum from
the maintainer. Quit older builds through the tray. Run the installer and open
Applifast from Start, or extract the ZIP and run `Applifast.exe`. Follow the
[Windows tester guide](docs/applifast/testing.md) for token import, Apple sign-in,
WebView2 setup, storage paths and troubleshooting. Tester ZIPs can include the
app developer token, so listeners only click Sign in with Apple. Public release
distribution and clean-machine acceptance remain pending. The installer uses
your user account, preserves settings on uninstall and does not register Spotify
links. Preview executables are unsigned. See the
[installer checks and limits](docs/applifast/windows-installer-evidence.md).
Maintainers can prepare optimized installer/ZIP downloads using the manual
[Windows preview workflow](docs/applifast/preview-downloads.md), once it is merged
and its signed developer-token secret is configured.

You need an Apple Music subscription, the Evergreen WebView2 Runtime and an
unexpired app developer token supplied in the ZIP by the maintainer. Keep
`developer-token.txt` beside `Applifast.exe`. Testers do not need a
Developer Program membership. Authorize your own subscriber account in Apple's
popup. Never share account credentials, user tokens or the `.p8` signing key.
If the playback host fails during startup, quit other Applifast builds and click
**Retry connection**. It reuses the saved setup; a missing or expired developer
token still needs renewal through the import control.

Developers can run from this checkout:

```powershell
cargo run --locked
```

For sample data without authorization or playback:

```powershell
cargo run --locked --features demo -- --demo
```

Apple song rows, the player bar, mini-player and selection menus offer Add to Favorites.
The mini-player replaces the inactive Apple EQ button with a heart; its
right-click menu also offers the action when rolled up.
Press **B** in either window to add the playing song. Already filled hearts
stay unchanged; typing and open dialogs suspend this shortcut.
Hearts update immediately; exact Apple confirmation retains successful songs
and rolls unconfirmed members back with a refresh-before-retry message. Removal
still requires Apple Music. This draft integration needs live-account testing.
Playlist, album and search reads now request Apple's explicit favorite flags
for song resources too, preserving each library/catalog identity. See the
[collection metadata evidence](docs/applifast/collection-favorites-evidence.md).

Windows saves now preserve open readers while replacing library, queue and
session files. Read-only files and locks that deny deletion still produce save
errors. See the [save evidence](docs/applifast/windows-save-evidence.md).

Choose **Play next** on a song, selection or album to hear it after the current
track, ahead of existing manual additions. **Add to queue** appends instead.
Albums load all track pages before insertion. See [Play next](docs/applifast/play-next.md).

Right-click an upcoming Apple queue row to **Remove from queue**. In the
mini-player playlist, select rows and use **REM > Remove selected**. This removes
only those queued copies, leaves the playing song alone and survives restart.
It never deletes library songs or edits an Apple playlist. **Remove all** still
clears only manually queued songs. See the [queue evidence](docs/applifast/queue-removal.md).

The Cargo package and build target retain the technical name `spotifast` and
version `0.12.0` during this port. Command help and `--version` now identify
`applifast`; older previews can still print `spotifast 0.12.0`. Use the source
revision from `BUILD.txt` for preview reports. The maintainer's portable package
names the executable `Applifast.exe`.
Token generation for maintainers is documented in the
[playback diagnostic guide](prototypes/apple-playback/README.md).

## Implemented in the preview

| Area | Current behavior |
| --- | --- |
| Library and search | Browse synced songs, albums, artists and playlists; search catalog and library with Load more. Original library and catalog identities remain separate. |
| Home | Recently played, Recently added, Heavy rotation and recommendations, with refresh and retry states. |
| Playback | Play/pause, seek, volume/mute, next/previous, shuffle and repeat. Unavailable uploads remain visible and are never silently replaced. |
| Queue | Play next, add, select upcoming rows, reorder, remove and clear. Repeated songs remain distinct occurrences. |
| Restart | Retain loaded song metadata and the local queue; restore playback paused and refresh in the background. Audio is streamed, without offline downloads. |
| Desktop | Tray, media keys, taskbar controls and mini player share the playback state. Closing to the tray can leave music running. |
| Playlists | Create playlists, save the queue and append selected songs, albums or the queue. Real-account write acceptance remains pending. |
| Favorites | Read flags, filter loaded songs, show a Favorites page and configurable sidebar shelf, and add up to 100 distinct non-favorite songs per action. Additions are locally tested; live-account acceptance is pending. Removal remains unavailable. |
| Links and menus | Copy public Apple Music catalog links or exact internal library links. Radio and advanced playlist edits are hidden. Playlist creation, additions and queue controls remain available. |
| Lyrics and motion | Optional LRCLIB lyrics and full-screen Now Playing; Reduce motion also stills settings switches, color fades and drag feedback; decorative Ambient Pulse. Apple playback does not expose PCM for EQ, spectrum, oscilloscope or MilkDrop. |
| Shortcuts | Settings > Keyboard shortcuts or Ctrl+/. Alt+1–5 navigates, F5 refreshes music pages and Ctrl+M opens the mini player. |
| Shared links | Paste an Apple Music share URL into Search and press Enter, or pass it to the executable. `play-uri` starts linked music explicitly; opening a link alone preserves playback. |

See the [listening-slice evidence and limitations](docs/applifast/listening-slice.md)
for implemented behavior, test results and remaining runtime gates. Mock/demo
checks do not establish real-account playback or clean-machine acceptance.

## Privacy and release readiness

Applifast keeps its settings, library/queue snapshot, window memory and browser
profile separate from Spotifast. Developer and user tokens use Windows Credential
Manager. Apple handles authorization, metadata and streamed audio; opening the
optional lyrics view sends song metadata to LRCLIB. There is no hosted backend,
app telemetry, decrypted-media cache or DRM circumvention. Upstream update checks
are disabled in Apple mode.

WebView2 contributes to the app's process-group memory and CPU. The reported
roughly 300 MiB footprint is an initial observation, not a verified resource
budget. Clean-machine authorization, full listening and gaming sessions,
network/output recovery, aggregate performance, installers and signed releases
remain acceptance work. The [tester guide](docs/applifast/testing.md) lists what
to exercise and how to report a result without secrets.

The [Windows resource sampler](prototypes/apple-playback/README.md#resource-measurement)
reports private memory for each app/helper process alongside the aggregate.
Use the desktop app's PID to include egui and WebView2 together. Short tooling
checks do not establish playback performance or pass the acceptance duration.

Report problems in [this fork's issues](https://github.com/jackkayser2005/applifast/issues).
Read [CONTRIBUTING.md](CONTRIBUTING.md) before proposing changes. Most legacy
pages under `docs/_guide/` and the inherited packaging instructions still
describe upstream Spotifast, not Applifast's Apple Music preview.

## Upstream credit

Applifast is a fork of [Spotifast by Carmine Paolino](https://github.com/crmne/spotifast).
It retains the original Rust/egui shell, desktop integrations and substantial
upstream implementation. Thanks to the Spotifast contributors and the
[fastframe](https://github.com/crmne/fastframe), [egui](https://github.com/emilk/egui)
and [librespot](https://github.com/librespot-org/librespot) projects. Librespot is
retained upstream code; Apple playback uses MusicKit/WebView2.

Applifast is independent and is not affiliated with Apple or Spotify. Apple
Music and Spotify are trademarks of their respective owners. Licensed under
the [MIT License](LICENSE), with upstream copyright preserved.
