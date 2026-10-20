# Hidden-host memory policy qualification

The embedded Windows playback renderer stays active while its native parent is
hidden, so MusicKit can initialize and continue playback. The earlier policy
treated that renderer as a visible browser and requested Normal memory use.
The candidate checks the actual native parent visibility and requests Low for
the hidden host. Visible diagnostic windows retain Normal. Showing/hiding the
native window now happens before choosing the memory target, which makes
reopened diagnostic windows return to Normal reliably.

This requalifies the policy from [the earlier experiment, PR #65](https://github.com/jackkayser2005/applifast/pull/65)
on the current integration. It uses the existing API and renderer lifecycle,
with the existing credentials, cache paths, network routes and dependencies.
Microsoft describes [the memory target](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_19)
as a best-effort hint. Scripts keep running, but memory can be paged out and
later access can slow down. A lower working set does not establish lower
private commitment. No memory-budget or latency saving is claimed here.

## Native policy regression

The actual WebView2 getter reproduces both problems. Before the policy change,
an active renderer with a hidden native parent reports Normal and fails the
check. With that policy fixed but the previous show ordering retained, the
first reopened diagnostic window reports Low despite being visible, and fails.
After both fixes, the native sequence is:

| State | Renderer active | Native parent visible | Requested target |
| --- | --- | --- | --- |
| Embedded-host fixture | Yes | No | Low |
| Diagnostic show | Yes | Yes | Normal |
| Diagnostic hide | No | No | Low |
| Diagnostic reopen | Yes | Yes | Normal |
| Diagnostic hide again | No | No | Low |

The permanent `test-memory-policy.ps1` regression checks this sequence twice,
including immediate profile reuse. Windows CI runs it after the ordinary host
and sampler checks. It waits for command acknowledgements before shutdown, so
stdin closure cannot suppress pending diagnostics. The fixture makes no Apple
navigation, credential read, authorization request or audio playback. It removes
its own disposable browser profile afterward.

On local Windows x64 with Evergreen `155.0.4283.62`, both cycles pass. Separate
cold/reused-profile setup checks pass in 0.335/0.300 seconds. These are component
setup durations, not first-song or click-to-play latency. Source and binary
hashes are retained with the qualification records.

## Current-source playback checks

The final Windows implementation has SHA256
`3537880bfdf78a10abdfe996c3aa60471813568a9228b09e4579d7529e13058b`.
Its default-feature test executable has SHA256
`a7191f02399c7bc4a1f2bdb9aa822b32dd2d1dcd20cc8d5ba57426de940e792b`.

The explicit read-only account check passes in 6.158 seconds: restored protected
authorization, HTTP 200, matching embedded page/bridge hashes, 100 library songs
with thumbnail metadata, ten catalog songs, and original/local/provider artwork
dimensions. The muted transport test passes in 56.389 seconds. It checks real
library shelves/details, library/catalog search, playback, seeking, pause,
saved-position queue resume, duplicate removal, Play next, and a 1,001-item
native context with manual duplicates and paused restoration.

Both use fresh owned profiles. The playback wrapper refuses a running normal
player and aborts its own test if that player starts. No playlist/favorite
mutations, sign-out or authorization popup is requested. These checks establish
host behavior, not audible frontend controls or UI timing.

The separate full-track check passes in 190.600 seconds. A complete 90.013-second
catalog song naturally advances to a manually added 90.013-second library song,
which also completes and advances to its context duplicate. The duplicate then
starts and pauses. No seek is used to shorten either track. Exact source/binary
hashes, before/after native policy states and runtime results are recorded in
[the qualification data](hidden-host-qualification-results.json).

Final local Windows checks pass: formatting; strict default/demo all-target
Clippy; default/demo all-target tests (1,038/1,084 library tests, eight opt-in
checks excluded per ordinary suite); doctests; strict demo Rustdoc; demo build;
gettext; Node bridge/token checks. The standalone host passes strict Clippy
and 26 tests, with its credential-store check excluded from the ordinary run.
The sampler fixtures and permanent native memory-policy regression also pass.
Hosted all-feature/platform checks still belong to the exact final feature head.

## Remaining runtime gates

Whole-player minimized playing/paused measurements, popup/retry behavior,
latency, audible desktop controls, output-device/network recovery and the
one-hour gaming session remain release gates. The maintainer deferred the
installed-player memory sample. Neither these native policy checks nor a lower
Task Manager working set closes the original 250 MiB aggregate-private limit.
The `2a3dd45` installer predates this candidate and remains available for testing;
this page is not a release claim.
