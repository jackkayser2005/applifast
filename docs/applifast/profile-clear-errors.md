# Sign-out browser cleanup errors

The native sign-out handler used to emit only `profileCleared: false` when
WebView2's asynchronous browser cleanup failed. Returning the native failure
from the callback did not send it to the app's error channel. A failure to start
cleanup instead reached the generic runtime-startup error, without sign-out
recovery guidance.

Both paths now use the existing fatal host-error channel. The app receives:

> Sign-out could not clear Apple browser data. Quit Applifast and reopen it
> before signing in again.

The host shuts down through its normal error path. The existing sign-out marker
and protected-grant deletion still happen before the browser command; startup
uses that marker to retry complete browser cleanup before restoring a session.
A failure message does not establish that cleanup succeeded. Successful cleanup
keeps its existing diagnostic and does not stop the host.

The regression injects a synthetic native failure into the production completion
handler. It fails before error forwarding and passes afterward. It checks the
fatal output, restart guidance, success behavior and exclusion of the native
error's private detail. It makes no Apple request, authorization popup, sign-out
command or real-account mutation. This establishes failure reporting, not actual
Apple cookie clearing or fresh-account sign-in.

There is no layout change, new dependency, storage format or network destination.
Actual profile clearing, popup cancellation/retry, revoked/expired credentials
and sign-out/re-sign-in remain [release gates](release-readiness.md).

Local Windows x64 checks pass: formatting; strict default/demo all-target Clippy;
all-target tests with 1,038/1,084 library tests and eight opt-in checks excluded
per ordinary suite; doctests; strict demo Rustdoc; demo build; gettext; and Node
bridge/token checks. The standalone host passes strict Clippy and 27 tests, with
its credential-store check run separately. Both native credential-store round
trips pass using temporary dummy entries. The native hidden-host policy fixture
also passes in its fresh profile without Apple navigation or audio. Hosted
all-feature/platform, site and Nix checks remain required on the final head.
