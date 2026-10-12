# Paused restoration and MusicKit states

The native transport expects its own playback states. Passing MusicKit's numeric
enum through unchanged made a restored, unloaded song look stuck loading.
The October 12 Windows runtime diagnostic confirmed `stopped=4`, `ended=5`,
`stalled=9` and `completed=10`. The temporary enum diagnostic was removed.

The bridge now converts those named SDK states at the shared boundary:

| MusicKit state | Native state |
| --- | --- |
| Stopped, with a current occurrence and paused intent | Paused (3) |
| Other stopped state | Idle (0) |
| Ended or completed | Finished (10) |
| Stalled | Waiting/loading (8) |

Other observed states keep their existing native interpretation. Transitioning
to another occurrence still reports loading. Queue identity, ordering, duplicate
occurrences and authorization generations are unchanged.

MusicKit can defer the physical seek until resume loads the restored media.
The paused display retains the requested position, while `actualPosition` still
reports the SDK's real time, including zero. No play call is added to restoration.
The real-account test now separately checks paused restoration and actual saved
position after resume, then pauses again. It retains both queue occurrences and
the manual-before-context order.

The JavaScript regression uses the observed enum values, checks their native
mapping and verifies that Next discards the old seek target. A Rust regression
checks that successive ended/completed notifications from an old request advance
only one occurrence. The original paused-state assertion failed before the fix.

The corrected ignored Windows test passed in 44.86 seconds with 100 library
rows. It restored protected authorization, read library/catalog resources,
played muted audio, sought, paused, restored two queue occurrences, resumed at
the actual saved position and paused again. The host reported paused (3) and
actual position zero before resume, then actual position at least 29 seconds
after resume. The first attempt stopped at a failed album-detail read; the
rerun passed without suppressing that error or adding production retries.

These checks used an owned workspace profile because of the previously recorded
execution-environment asset-location failure. No other running app/profile owner
was present. No Favorites/playlist writes, authorization popup or Sign out command
was sent. The protected grant was retained; disposable browser profiles are removed
after shutdown. This is native host acceptance, not a real desktop restart test.

Windows formatting, strict default/demo all-target Clippy, 1,017 default and
1,055 demo library tests plus remaining targets, doctests, strict Rustdoc,
demo build, gettext and Node bridge/signer checks pass. Five opt-in tests were
ignored in each ordinary library run; the native result above is separate.
All-feature and other-platform validation still require the fix's hosted CI.

This changes playback reporting, without changing layout, credentials, files,
network destinations, dependencies or media storage. Full-track listening,
real application restart, fresh-PC authorization and gaming acceptance remain
separate items on the [release checklist](release-readiness.md).
