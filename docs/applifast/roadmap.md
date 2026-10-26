# Windows release priorities

This orders work toward a Windows beta and then 1.0. All eleven areas in the
[acceptance checklist](release-readiness.md) remain in scope. Implemented code,
passing mocks and cross-platform compilation do not replace runtime acceptance.

## P0: memory and listening reliability

| Work | Evidence needed |
| --- | --- |
| Native memory | Profile equivalent fresh-start, library, lyrics and main/mini/tray transitions. Fix demonstrated retention and measure before/after. |
| Playback-host overhead | Validate the minimal document and any low-memory candidate with real authorization, reads, full tracks, seeking and mixed queues. Count every helper. |
| Gaming budget | Two-minute warm-up, five-minute minimized playing/paused samples: at most 250 MiB aggregate private memory, under 1%/0.2% average total CPU, then a one-hour gaming session. |
| Family test | Dad installs the provided installer, authorizes his own account, loads his library and hears complete tracks. Record source revision, Windows/runtime version and failures. |
| Account recovery | Real popup close/cancel/retry, expiry/revocation, sign-out/profile clearing and reauthorization. Pending work and late events cannot restore an old account. |
| Listening recovery | Network loss, output-device changes and queue failures retain the selected song and occurrences, with usable retry. Resolve intermittent native Next failures. |

The minimal hidden page is [PR #69](https://github.com/jackkayser2005/applifast/pull/69).
The [native investigation](native-memory.md) adds window-artwork and interrupted
theme-reveal cleanup, plus an isolated font-storage prototype that saves about
59 MiB of private memory in matched demo runs. A [credited upstream backport](font-memory.md)
now has a maintainer-owned revision pin; app checks and package qualification
remain separate from the initial prototype. Measured impact on the complete live
player remains required. The family's `63d85dd` installer predates these changes.

## P1: verify the supported Apple experience

| Work | Evidence needed |
| --- | --- |
| Library/catalog | Representative pagination/search, Home recently added/history/recommendations, cache refresh and restart. Preserve library/catalog IDs and visible unavailable uploads. |
| Queue/controls | Play/pause/seek/volume, next/previous, shuffle/repeat, duplicates, Play next, Add to queue, drag/reorder/remove/clear, upcoming-row selection, restoration and stale-event protection. |
| Desktop surfaces | Audible main/mini/tray recreation, taskbar controls, media keys and reopen. Retain settings and correct window geometry across restart. |
| Favorites | Confirm new writes in Apple Music from every exposed surface, rollback, page/filter and configurable left-sidebar shelf. Unsupported removal stays absent. |
| Playlists | Confirm create and append tracks/albums/queue, duplicates, permissions and delayed visibility. Ambiguous failures cannot cause duplicate writes. |
| Motion/lyrics/settings | Exercise all eleven PR #13 items, Reduce motion, Ambient Pulse and LRCLIB success/missing/error states. Inspect light/dark, narrow/normal and actual animation timing. |
| Product identity | Audit executable/window/installer/storage/credentials/updater and every menu. Hide unsupported Spotify/PCM controls and retain explicit non-Windows playback status. |

Adapted PR #13 and the previously authorized visual scope remain part of the
[integration](integration.md). Still images do not prove animation behavior.

## P2: dependable distribution

| Work | Evidence needed |
| --- | --- |
| Source/checks | Assemble reviewed slices linearly with upstream credit. Pass applicable platform/all-feature, credential, docs, packaging and Nix checks on the release source. |
| Installer/upgrades | Optimized installer/ZIP, clean-PC startup, WebView2 setup, shortcuts, upgrade/uninstall, publisher-signing decision and checksums. |
| Token delivery | Listener needs no developer membership. Test expiry and renewal without shipping a private key. Family-preview token expires November 6, 2026. |
| Documentation/privacy | Install/sign-in, capabilities, storage/cache, network access, troubleshooting and reporting. Recheck ignored secrets, source history and payloads. |
| Publication | Version/lockfile/release metadata/Nix hash and written notes, required checks, tag, verified artifacts and links. Keep inherited upstream publishing guarded. |

The invited installer remains a private preview. Publish a Windows beta after
its applicable acceptance gates pass, collect tester results and resolve the
remaining regressions before declaring Windows 1.0. Do not label failed resource
limits as passes.

The checklist's post-release boundaries still apply to true PCM visualizers/EQ,
native DRM and unsupported advanced playlist editing. Decorative motion is not
audio analysis. Keep MIT credit, no telemetry/backend, no decrypted-audio cache
and no DRM circumvention.
