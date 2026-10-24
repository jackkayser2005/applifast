# Apple reads that ignore cancellation

The existing 20-second read deadline previously asked MusicKit to abort, then
continued awaiting its promise. An SDK that ignored the signal could leave a
library/page read or Favorites confirmation pending indefinitely.

The bridge now races each API operation against local cancellation. Read expiry
aborts the underlying signal and releases the bridge's wait independently of SDK
cooperation. Sign-out does the same for pending API operations. Timers and abort
listeners are released on success, failure or cancellation. Late fulfillment or
rejection remains observed by the race and cannot publish a second response.

The existing error and Retry controls are unchanged. Library/page failures retain
cached data, and a failed Favorites confirmation returns an unknown flag with
refresh-before-retry guidance. Canceling a POST locally does not prove Apple
canceled or reverted it. Writes retain their existing lack of an automatic
deadline or retry. Pending old-account writes do not start after sign-out.

There are no new dependencies, endpoints, credentials, storage files or schema
changes. The helper runs inside the independent playback host, without blocking
the egui thread. Unsupported non-Windows playback remains unchanged.

## Regression evidence

The test deliberately uses an SDK promise that ignores its abort signal. Against
the parent bridge it fails with `SDK ignored abort: library`, reproducing the
unreleased wait. After the change it passes library and page deadlines, Favorites
confirmation expiry, sign-out during a read and sign-out during a playlist POST.
Every case covers both late success and late failure. Transport pause remains
responsive, request counts prove no automatic retry, old-account queued writes
do not start, and a late result cannot publish success or leak SDK error text.

These are deterministic bridge checks, without Apple authorization, real account
writes or audio. They do not establish a live-service timeout, cancellation at
Apple, clean-PC sign-in, browser-profile clearing or gaming performance. No
appearance or control layout changes, so the existing
[API recovery comparisons](review-api-recovery/index.html) describe the error UI;
still captures cannot demonstrate a promise being released at its deadline.

Windows formatting, strict default/demo all-target Clippy, all-target tests,
doctests, strict demo Rustdoc, demo build, gettext consistency and Node
bridge/signer checks pass. The library suites pass 1,026 default and 1,071 demo
tests, with six opt-in checks ignored in each. Standalone-host formatting,
strict all-target Clippy and 18 tests pass, with one native credential test
ignored. Credentials did not change in this patch. All-feature, site and Nix
verification require the hosted jobs; real-account playback is not rerun while
the installed player owns its profile. CI and optimized-package outcomes are
recorded separately in the feature PR.
