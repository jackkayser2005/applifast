# Native WebView2 setup deadline

Environment creation, main-controller creation, profile clearing and bootstrap
script registration now share one 45-second asynchronous setup deadline. The
previous callback waits could block indefinitely before the sign-in card became
ready. Expiry returns the fixed error "WebView2 setup timed out. Check the
Evergreen runtime and retry the connection." through the existing error/Retry
path. It does not delete saved setup, credentials, account snapshots or queues.

The wait continues dispatching Windows messages on the existing dedicated STA.
It checks the same deadline between callbacks, preserves native failure HRESULTs
and recognizes cancellation or a disconnected completion channel. It does not
add a polling loop during ordinary playback. Microsoft documents the required
[STA and message-pump ownership](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/threading-model)
and [finite message-aware waits](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex).

Owned guards close the main controller/window and uninitialize COM on setup
failure. A controller delivered after its receiver closes is also closed on the
owning thread. Explicit normal shutdown still reports controller/window-close
errors. The existing popup path remains asynchronous, outside these setup waits.

The separate JavaScript SDK-load deadline remains 45 seconds. Native setup's
deadline covers asynchronous completion waits, not an Apple authorization popup,
page navigation, SDK configuration or every synchronous Windows/filesystem call.
This is not a total application-startup time guarantee. No dependencies,
credential format, storage location, endpoint or app telemetry changes.

## Evidence

The extracted wait initially retained the existing unbounded pump. Its native
regression fails because it does not finish when no callback arrives. A late
completion releases that old worker so the failing test leaves no thread behind.
With the finite wait, the same regression passes and rejects late delivery.

Additional real Windows fixtures dispatch a completion through an owned hidden
window, preserve an error HRESULT, handle a disconnected channel and WM_QUIT,
then verify an expired setup destroys its window and releases the STA before
another apartment/window is created. These fixtures use no Apple credentials,
playback host, user window or account mutation.

The standalone host passes strict all-target Clippy and 21 tests, with one
native credential check ignored in the ordinary suite. Its actual Evergreen
self-check also starts and shuts down twice against one isolated initially empty
profile, including immediate reuse. Both return exit 0 with runtime/hostReady
events in 0.518 and 0.355 seconds. Self-check reads no Apple grants and does not
navigate to an Apple page, authorize, play audio or write to an Apple account.
The existing installed player is left alone.

These checks prove native wait/lifetime behavior and successful local runtime
setup. They do not establish a real delayed WebView2 callback, fresh-PC Apple
sign-in, audible playback, profile clearing after real sign-out or gaming
performance. The visible change is diagnostic text in the existing native
sign-in error card; its layout and controls are unchanged. Native still-image
and browser-rendered error-state comparisons remain a visual acceptance step.
The feature PR records the full app checks, hosted CI and optimized package.

Windows formatting, strict default/demo all-target Clippy, all-target tests,
doctests, strict demo Rustdoc, demo build, gettext consistency and Node
bridge/signer checks pass. Library suites pass 1,026 default and 1,071 demo tests,
with six opt-in checks ignored per ordinary suite. Standalone formatting and
strict all-target Clippy pass too. This patch does not change credential storage.
Live playback is not rerun while the installed player owns its account profile.
All-feature, site, other-platform and Nix checks remain subject to hosted CI.
