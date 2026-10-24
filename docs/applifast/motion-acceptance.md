# PR #13 motion acceptance ledger

All eleven requested paths remain in scope. Source implementations and static
comparisons do not prove every native transition or real-account interaction.
The evidence below qualifies only the named path and conditions.

| Requested item | Current evidence | Remaining acceptance |
| --- | --- | --- |
| Reduce motion in Settings | Connected setting, switch/tint/drag regressions and [setting comparisons](reduce-motion-controls.md). Native indicators stay still; native hover snaps directly to its final state. | Actual Settings interaction alongside every remaining transition. CLI-selected demo settings do not establish that interaction. |
| Play on cover hover, with lift | [Native hover frames](native-hover-frames.md): intermediate native pixels, four-pixel lift and Play affordance, both themes and widths. Existing deterministic pointer/action tests cover dispatch. | Natural pointer/click use with the Apple library. |
| Dancing bars on the playing song | [Native indicator frames](native-motion-frames.md): eight distinct moving frames and identical Reduce motion frames in each theme/size. | Live playing/paused/zero-volume desktop use. These are decorative patterns, not audio analysis. |
| Thicker seek bar and hover knob | [Native hover frames](native-hover-frames.md): intermediate fill colors, thicker track and visible knob; Reduce motion snaps. Deterministic keyboard/pointer controls are tested. | Natural hover/seek while real Apple playback continues. |
| Cover crossfade | Connected state machine and ready/pending/rapid-skip tests. [PR #91](https://github.com/jackkayser2005/applifast/pull/91) reproduces and fixes failed-art retention, with actual paint regressions and sixteen native end-state captures. | Native success-transition frames and natural image arrival/failure during listening. PR #91's static captures do not establish timing. |
| Heart pop | Connected player/row/mini-player callers, optimistic filled-heart scale fixes and focused paint/state tests. | Native temporal frames and real eligible favorite actions. New account writes still require the chosen song's authorization. |
| Queue rows slide in | Occurrence-keyed arrival path, deterministic occurrence/order/cancellation tests and queue comparisons. | Native temporal frames for new occurrences, plus real additions and stale responses. |
| Readable art-colored header | Clamped palette helpers and tint/Reduce motion paint regressions. | Native temporal frames and contrast/readability across representative real artwork. |
| Full-screen Now Playing controls | Connected actions, deterministic full-screen/lyrics layout checks and static comparisons. | Audible transport controls, main/mini/window recreation and settings interaction. |
| Smoother synced lyrics | Motion-aware timed line/scroll drawing and deterministic word/follow/layout tests; Apple uses LRCLIB when available. | Native temporal frames and natural follow/seek/manual-scroll behavior, including missing/error lyrics. |
| Short page fade | Connected page-opacity path and deterministic transition/Reduce motion tests. | Native temporal frames and natural navigation without flicker or stalled interaction. |

Current hover evidence uses paused sample data, no image URLs and a temporary
capture marker. The existing native indicator evidence uses its documented demo
conditions. Neither establishes memory limits, one-hour gaming reliability,
account lifecycle, new Favorites writes or all eleven paths at once.

The [source integration trace](integration.md) and complete
[release checklist](release-readiness.md) remain authoritative for broader scope.
