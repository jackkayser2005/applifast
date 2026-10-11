# Favorites write integration gate

Baseline: `0cdfe6b235159796d4ec2cb16c70d36380356e5a` (PR #37).
Windows investigation: October 11, 2026. This is unfinished feature work.

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

The current native attempt failed before MusicKit initialization with WebView2
HRESULT `0x8007139F`. No favorites request or audio command was sent. Evergreen
runtime `154.0.4258.62` is installed. The existing `e501564` preview, PID 38016,
still owns an Applifast WebView2 process group using the dedicated profile.
Microsoft identifies this error as incompatible options in a shared browser
process. The running build is a likely conflict, not a proven cause until a
check with that build closed succeeds:
[WebView2 environment errors](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl).
The user was asked to quit through the tray; their app/profile was not stopped,
deleted or modified by this investigation.

After that gate, remaining work includes optimistic desktop controls, partial
and failed write confirmation/rollback, stale-read protection, bulk interaction,
restart behavior, a previously unfavorited song, matching visual comparisons
and a new tested download. Removal remains unimplemented without a verified
supported route. Neither the full favorites feature nor release readiness is
claimed complete.
