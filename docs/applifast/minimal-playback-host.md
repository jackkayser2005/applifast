# Minimal native playback document

The Windows app previously loaded the standalone diagnostic page inside its
independent WebView2 host. That page kept a second song list alongside the native
egui library. Every library response appended rows, including repeated refreshes,
and rebuilt all diagnostic song buttons. Playback-state events also updated its
hidden status text.

The embedded host now loads `web/host.html`, containing only the shared bridge
and Apple's MusicKit JS v3 script. The standalone probe still loads
`web/index.html` with its diagnostic controls. The existing embedded-host flag
selects the document. Ordinary screens and their layout remain in egui; the
embedded document title becomes `Applifast`.

## Measured baseline

The maintainer reports roughly 260 MB in the desktop process view. A separate
read-only sample of the installed, hash-verified `63d85dd` preview records:

| Measurement | Result |
| --- | ---: |
| Sample duration | 30.098 seconds |
| Snapshots / process identities | 30 / 9 |
| Peak aggregate private memory | 713.992 MiB |
| Native app private memory at that peak | 419.301 MiB |
| Eight WebView2 processes at that peak | 294.691 MiB |
| Native resident working set immediately after sampling | 271.68 MiB |

The native executable matches SHA256
`1398ed1d79ff0068c0d5ed745444dc1d53620e287789e199bb03a1e7a2dfea70`.
WebView2 uses Evergreen `155.0.4283.62`. Descendants were selected from the
Applifast root process, excluding Windows Search's unrelated WebView2 processes.
The raw local report has SHA256
`38655ae39372b27a8ee8a6c6ff717db183d4b4025778fe447f2f9d95c35e7a28`.

Private committed memory and resident working set are different measures. This
does not establish which Task Manager column the maintainer observed. The app
was left in normal use; no playback, window or account command was sent. There
was no controlled warm-up, minimized state or verified playing/paused state.
The sampler correctly records `acceptance_duration_not_met`. This is a short
allocation breakdown, not a resource-budget pass or evidence of a leak.

The native process is the largest contributor. Its artwork loader has a 64 MiB
accounting budget for compressed/decoded artwork and textures, but this process
sample does not identify how much of that budget is occupied or attribute the
remaining allocations. Native image/font/renderer retention needs allocation
profiling and a repeatable fresh-start, navigation and window-recreation scenario.
Do not attribute the entire footprint to WebView2 or assume this page change
alone reaches the 250 MiB aggregate target.

## Regression and verification

- A Rust regression fails against the old embedded page because it includes
  the diagnostic library list, then passes with the new page selected.
- A Node fixture executes the actual page scripts and shared bridge through ten
  repeated 100-song responses. Both pages deliver 1,000 rows to the native
  callback. The probe retains 1,000 diagnostic rows and allocates 11,000 DOM
  nodes cumulatively; the native page creates zero diagnostic nodes. These are
  fixture counts, not measured MiB or real MusicKit playback acceptance.
- Local Windows formatting, strict default/demo all-target Clippy, default/demo
  all-target tests, doctests, strict demo Rustdoc, demo build, gettext and Node
  bridge/signer checks pass. Library suites pass 1,029/1,075 tests with seven
  opt-in checks ignored per suite. The standalone host passes strict Clippy and
  25 tests, with its native dummy credential-store round trip ignored.
- All seven hosted jobs pass on the preceding `63d85dd` source. They do not
  validate this follow-up. Its own hosted checks remain required.
- The current private installer predates this page change. Real Apple sign-in,
  restored authorization, library delivery, playback/seek/queue behavior and
  memory measurement on the minimal document remain required before a new
  qualified preview. The maintainer's running player was not interrupted.

Repeat the existing two-minute warm-up and five-minute minimized playing/paused
samples on the candidate, including all associated processes. Compare equivalent
actions and runtime versions before claiming savings. Keep the 250 MiB memory,
1% playing CPU and 0.2% paused CPU limits, plus the one-hour gaming trial.

## Storage and network

The app still writes its compiled document and bridge to the existing playback
profile's `assets/index.html` and `assets/bridge.js`. The local virtual URL,
profile, protected credentials, origin validation, popup handling, STA ownership
and native command/event channels remain unchanged. It still loads Apple's
official SDK from `js-cdn.music.apple.com`; no new endpoint, dependency,
settings format, telemetry or audio persistence is added. The page does not
change MusicKit's own queue or internal caches.
