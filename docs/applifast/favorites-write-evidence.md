# Favorites write integration gate

Baseline: `0cdfe6b235159796d4ec2cb16c70d36380356e5a` (PR #37).
Windows investigation: October 11, 2026. This is unfinished feature work.

The host-boundary history below predates the draft
[desktop add-only integration](favorite-controls.md). Its original read-only
statements describe those baselines; current desktop controls and their remaining
acceptance gates are documented separately.

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
