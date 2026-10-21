# Recover when the native playback host stops

A native host failure could leave the app looking authorized and ready even
though its command receiver had gone away. Retry then sent authorization to the
stopped host. A queued ready event could also revive that stale state before
the listener clicked Retry. Separately, an early service error could return
before the native STA was shut down, keeping its browser profile in use.

Trusted Rust host errors now mark the host unavailable and invalidate its
generation immediately. Ordinary SDK/API operation errors keep the live host.
The existing Retry connection action creates a replacement through the backend;
repeated clicks while startup is pending do not create another. A native thread
owner sends shutdown, wakes the STA and joins it on both ordinary and early
service exits. This work stays off the UI thread.

Before invalidating authorization, the app saves the latest account-bound
metadata/session snapshot using the existing atomic local-cache write. It
clears transient loading, search and pending-write views. For a verified matching
account, the existing cache path restores duplicate occurrences, manual-before-
context order and position, paused. A different or unverified account tag clears
retained contexts before new account data is accepted. Uncertain playlist or
favorite writes are not replayed. Sign-out keeps its existing cache clearing.

No new dependency, credential key, storage format or network destination is
introduced. The browser cannot set the trusted fatal flag through its sanitized
bridge. Non-Windows playback remains explicitly unsupported. The existing
sign-in card and reviewed layout are unchanged.

## Regression evidence

- The original App test failed because a fatal error left account readiness
  true. A second failing assertion reproduced the late ready event reviving it.
  The corrected tests cover both failure paths, snapshot disk round trip,
  three occurrences of the same song, a manual insertion, position restoration
  and duplicate Retry clicks. On Windows, the test draws the real existing card
  and clicks Retry connection through its accessibility action, without a
  desktop window or real host.
- The different-account regression first verifies its private playlist fixture
  is present. Its corrected pre-fix assertion failed when that playlist survived
  authorization as another account. The fixed check also verifies that a
  retained playing context clears only after the new account is verified.
- The native owner fixture creates an actual COM STA and Windows message queue,
  returns a synthetic service error and verifies shutdown/join. Its pre-fix
  owner leaked the worker until the test's fallback released it; the fixed
  owner releases it directly. It uses no WebView, credential or Apple request.

Standalone strict Clippy and all 28 ordinary host tests pass on Windows. The
isolated Evergreen checks preserve the low-memory policy, synthetic cookie
cleanup and cleanup-before-authorization ordering. The dummy-grant Credential
Manager round trip also passes. A separate saved-account read-only check loads
100 library songs and ten catalog songs in 5.949 seconds, with matching embedded
page/bridge assets and no audio, popup, sign-out or account writes. Its owned
profile and the completed focused-test fixtures were removed afterward.

Local default/demo checks pass: formatting, strict all-target Clippy, all-target
tests (1,040/1,086 library tests, eight opt-in checks excluded per ordinary
suite), doctests, strict demo Rustdoc, demo build, gettext and Node bridge/token
checks. Translation changes only refresh source references and the extraction
timestamp. All-feature,
Linux/macOS, site and Nix checks still require hosted CI on the feature head.
The local Linux launcher check could not run because Ruby and its POSIX utility
environment are unavailable; the all-feature Windows toolchain also lacks vcpkg.

These deterministic and component checks do not establish an actual account's
audible recovery after a host failure. Fresh-PC sign-in, real Apple popup and
sign-out behavior, expiration/revocation, minimized resource measurements and
the gaming soak remain release gates. The ordinary-player measurement was
deferred at the maintainer's request.
