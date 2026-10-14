# Favorites write integration gate

Baseline: `0cdfe6b235159796d4ec2cb16c70d36380356e5a` (PR #37).
Windows investigation: October 11-12, 2026. This is unfinished feature work.

The host-boundary history below predates the draft
[desktop add-only integration](favorite-controls.md). Its original read-only
statements describe those baselines; current desktop controls and their remaining
acceptance gates are documented separately.

## Current native check, October 12

The opt-in no-audio account check now passes with an isolated workspace browser
profile and the existing authorization restored from Windows Credential Manager.
The unchanged host loaded its local page with HTTP 200, reported authorized
MusicKit readiness and read Apple's library/catalog metadata. It selected an
already-confirmed library favorite and an independently confirmed catalog
favorite, sent each idempotent request and verified exact resource types, IDs
and `inFavorites=true` in the uncached confirmation and subsequent reads.
The complete native check passed in 6.51 seconds after compilation.

The first attempt exposed an invalid fixture assumption: one library song was
already a favorite while its catalog counterpart reported false. The test now
searches the first library page's linked catalog candidates independently,
using existing validated individual-song routes. It requires both fixtures to
report true before either write. The passing run considered a set of 30
candidates and found a confirmed catalog favorite. The protocol boundary was
not broadened for the test. No song was newly favorited and no audio, removal,
authorization-popup or Sign out command was sent.

The existing-profile failure was narrowed separately. Temporary diagnostics
showed that the failed completion belonged to the expected local-page navigation
and was not cancelled. Both on-disk asset hashes matched the source. Serving
identical assets from the workspace with that same browser profile reached
MusicKit readiness. The temporary hardcoded path and navigation diagnostics
were discarded; production profile/asset paths remain unchanged. The execution
environment's asset-location failure is not fully explained.

Before testing, process checks found no running app or browser-profile owner.
For the passing account check, only the child test process's `LOCALAPPDATA`
selected the disposable profile. It did not copy another account's browser
profile or export a listener token. The profile was removed after the result
was recorded, without clearing the protected grant. The standalone diagnostic
binary was rebuilt after discarding temporary source changes.

This verifies native authorization restoration, reads and idempotent Favorites
request/confirmation behavior. It does not verify adding a previously unfavorited
song, desktop optimistic interaction, fresh-PC sign-in, full-track playback or
gaming performance. The [release checklist](release-readiness.md) retains those
gates. The earlier failed attempts and historical implementation stages below
remain useful context, not the current account-test result.

## Muted playback gate, October 12

The existing opt-in playback test also ran with a disposable workspace profile,
protected authorization restoration and no other app/profile owner. Library,
playlist, album and artist reads, detail routes and library/catalog search
completed. Muted playback advanced beyond one second, seeking reached at least
29 seconds and pause was confirmed. No playlist or Favorites write was sent.

The complete test still failed. Its final paused-restoration assertion timed out
after a total 59.70 seconds. Safe numeric diagnostics reported generation 4,
SDK status 4 (classified as loading by the native app), requested position 30, actual position 0, duration 0 and
two queue entries. Earlier playback/seek/pause assertions had already passed.
The test now retains those numeric fields on timeout without logging account
identifiers, song metadata or authorization responses. Restoration needs further
investigation; this result does not establish restart recovery or full-track
listening. The disposable profiles were removed after shutdown.

A later public-enum diagnostic identified SDK status 4 as `stopped`, not loading.
The [state-conversion fix](paused-restoration.md) addresses that mismatch and
extends the native test to verify the actual saved position after resume.

## Boundary

The host now accepts a typed `favoriteSongs` command with a JavaScript-safe
request ID and 1 to 100 validated catalog/library song items. This local batch
bound is not presented as an Apple API limit. The MusicKit request uses
`POST /v1/me/favorites` with typed ID parameters and no request body. Repeated
IDs are deduplicated within each resource type; original resource IDs survive
even when playback or catalog-match IDs differ. No rating or removal endpoint
is used. Playback queues and their duplicate occurrences are untouched.

The command uses the existing write channel, independent of playback commands.
Writes are not automatically retried or given a new timeout, because an
interrupted mutation may have reached Apple. Sign-out aborts in-flight requests,
discards queued writes from the old session and suppresses late responses.
Errors use fixed messages instead of raw SDK responses. This adds no dependency,
credential format, app setting, network destination or media persistence.

Apple documents a 202 empty response and warns that unsupported requested IDs
may be ignored. Acceptance alone cannot establish that every song changed:
[add resource to favorites](https://developer.apple.com/documentation/applemusicapi/add-resource-to-favorites).
The exact typed-ID route still needs real-account verification. Desktop hearts
remain read-only, with no layout or visual changes in this gate.

## Per-song confirmation

The diagnostic `favoriteSongs` command now follows an accepted POST with one
batched GET per requested resource type, in parallel. Catalog IDs use the active
validated storefront; library IDs use the original library endpoint. Both reads
request `extend=inFavorites` and disable HTTP cache reuse. These use Apple's documented
[catalog song](https://developer.apple.com/documentation/applemusicapi/get-multiple-catalog-songs-by-id)
and [library song](https://developer.apple.com/documentation/applemusicapi/get-multiple-library-songs)
routes. No catalog-match ID or playback ID replaces the requested identity.

The response's `data.data` contains one minimal resource per unique requested
typed ID. A single exact matching resource with a boolean `inFavorites` supplies
that flag. Missing, wrong-type, substituted, duplicate, malformed and failed
read results stay unknown, represented by an empty attributes object. Any false
or unknown flag produces a fixed error alongside the per-song data, preserving
confirmed members of a partial result. API credentials and unrelated response
fields are excluded. An empty 202 body alone no longer reports confirmation.

Confirmation reads retain the existing 20-second deadline. They do not block
transport controls, repeat the mutation or automatically retry reads. Apple may
still be updating the library when this immediate read happens; an unconfirmed
result asks for a refresh before retrying the write. Sign-out aborts active
reads, suppresses late confirmations and prevents a stale completed POST from
starting new reads. No dependency, persisted format, setting or destination is
added. Desktop optimistic controls, rollback and bulk interaction remain pending.

## Checks and limits

Rust boundary tests cover round-trip identity, mismatched library playback IDs,
invalid IDs, oversized/empty batches, unsafe request IDs and exclusion from
playback intents. Node tests cover mixed catalog/uploaded resource parameters,
deduplication, empty responses, rejection before network access, responsive
Pause, sanitized failures and queued-write cancellation after sign-out.

Windows formatting, strict default/demo all-target Clippy, 1,009 default and
1,043 demo library tests (five opt-in tests ignored in each), binary/integration
suites, doctests, strict demo Rustdoc, demo build, gettext and Node bridge/signer
checks passed. The standalone host passed formatting, strict all-target Clippy
and 14 ordinary tests (one native-store check ignored). These local checks do
not cover optional projectM, Linux/macOS runtime, Jekyll or Nix.

The opt-in Windows check requires existing local authorization and two already
confirmed favorites. It reads their exact state before re-adding either and
confirms exact IDs afterward, without audio. Even a passing result would only
prove that these idempotent requests were accepted and the existing state was
retained, not that adding a new favorite works or that unsupported IDs change.

The initial native attempt failed before MusicKit initialization with WebView2
HRESULT `0x8007139F`. No favorites request or audio command was sent. Evergreen
runtime `154.0.4258.62` is installed. The existing `e501564` preview, PID 38016,
still owns an Applifast WebView2 process group using the dedicated profile.
Microsoft identifies this error as incompatible options in a shared browser
process. The running build is a likely conflict, not a proven cause until a
check with that build closed succeeds:
[WebView2 environment errors](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl).
The user was asked to quit through the tray; their app/profile was not stopped,
deleted or modified by this investigation.

After the user quit that preview, the October 11 check no longer produced
`0x8007139F`. WebView2 reached `hostReady`, but MusicKit did not become ready
within the existing 45-second deadline. A credential-free host startup/shutdown
check passed on runtime `154.0.4258.62`; an independent HEAD request to Apple's
public SDK returned HTTP 200. Neither establishes SDK loading in WebView2.

The host now waits for bootstrap-script registration before navigation, as
[Microsoft requires](https://learn.microsoft.com/microsoft-edge/webview2/reference/win32/icorewebview2#addscripttoexecuteondocumentcreated).
This corrects an asynchronous ordering bug but did not resolve this machine's
native gate. Navigation diagnostics then isolated the failure to the local
playback page: `success=false`, WebView2 status `0`, HTTP status `0`. The fixed
page-loading error now appears immediately instead of leaving setup waiting.
Diagnostics contain only fixed stage names, booleans and numeric statuses, never
URLs, page contents, credentials or SDK responses. The test now identifies its
awaited stage. Native folder-path separator normalization did not change the
result and was discarded. The asset files exist, and no explicit deny rules were
found on their ancestors; the remaining cause is unresolved.

No favorites request or audio command was sent in these attempts. The
real-account favorites gate remains failed, and successful native reconnection
must not be inferred from the green CI checks or the absence of the earlier
profile-conflict HRESULT. The maintainer confirmed normal library loading in the
`2d0ca87` desktop preview after these attempts. That baseline's library view may
include cached data; it does not establish favorites writes, identify the
automated failure's cause or verify a newer desktop build.

The maintainer subsequently confirmed library loading and audible catalog
playback in the optimized `9cb3a36` desktop preview. This establishes normal
desktop playback separately from the failing automated host check. It does not
establish full-track completion, fresh-account sign-in, favorite writes or the
gaming resource budget. The confirmation changes described above are newer
than that preview and have not been tested against the account.

The known failed-page case was separately checked: the host emits the exact
fixed retry error and exits with failure instead of hanging, with no account
requests or audio. This is negative-case verification, not a passing favorites
gate. Full available Windows formatting, strict default/demo Clippy, 1,011
default and 1,045 demo library tests (five opt-in tests ignored in each),
binary/integration suites, default doctests, strict demo Rustdoc, build, gettext
and Node bridge/signer checks pass after the change. Standalone host formatting,
strict Clippy and 15 ordinary tests pass. Final-head hosted CI remains required.

For per-song confirmation, the same available Windows checks pass: 1,011 default
and 1,045 demo library tests (five opt-in tests ignored in each), the remaining
binary/integration suites, formatting, strict Clippy, doctests, strict demo
Rustdoc, demo build, gettext and Node bridge/signer checks. The standalone host
passes formatting, strict Clippy and 16 ordinary tests (one native-store test
ignored). Regressions exercise false/missing/malformed/ambiguous confirmations,
partial read failure, deadlines, transport responsiveness and sign-out both
before and during confirmation. No real-account favorites test has passed.

After that gate, remaining work includes optimistic desktop controls, partial
and failed write confirmation/rollback, stale-read protection, bulk interaction,
restart behavior, a previously unfavorited song, matching visual comparisons
and a new tested download. Removal remains unimplemented without a verified
supported route. Neither the full favorites feature nor release readiness is
claimed complete.

## Isolated startup follow-up, October 12

A separate startup check used a fresh browser profile under the worktree's
ignored `.cache/` directory, selected through the child process's `LOCALAPPDATA`.
A `signed-out` marker was created before startup so the host did not restore
the stored listener token. It used the already-valid developer token from
Credential Manager, without importing or replacing credentials. The only
command sent to the host was shutdown; the running desktop app was left alone.

The local page reported `success=true`, WebView2 error status `0` and HTTP
status `200`, followed by MusicKit `ready` and process exit `0`. The diagnostic
output retained only these fixed stages and numeric statuses. This establishes
local-page navigation and SDK readiness with the isolated signed-out profile.
It does not establish fresh-account authorization, account reads, audio or
Favorites writes. The earlier existing-profile failure remains unexplained;
it is not a universal failure to load this page in the automation environment.

No source change was needed for this check. The existing `--self-check` still
only creates the WebView2 controller and reports the runtime version; it does
not navigate to the playback page or test MusicKit. The
[current release checklist](release-readiness.md) tracks the remaining
real-account and private-trial gates separately from this startup result.
