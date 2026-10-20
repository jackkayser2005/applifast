# Finish browser cleanup before a new authorization

Sign-out started asynchronous WebView2 profile cleanup and immediately continued
draining native commands. A new authorization in that batch could therefore run
before cleanup completed. The real native fixture reproduces this on the
`afcded9` production base, with its new test/diagnostic instrumentation present:
`authorize, cleared, authorize, cleared`.

The native host now holds later dispatched commands until the cleanup callback
succeeds. Callback completion wakes the existing STA message loop, which drains
the bounded pending queue. It rechecks readiness for each command, including
commands already taken into the current batch. The SDK's existing sign-out
cancellation/transport sequencing stays in place. Show/hide and shutdown remain
outside the dispatch wait; no UI-thread or native message-pump wait is added.

Failed cleanup retains the fatal-error behavior from [the preceding fix](profile-clear-errors.md).
A native 45-second timer also stops the host with restart guidance if completion
does not arrive. The sign-out marker remains available for the next startup to
retry cleanup. This timer's failure path has not been exercised against an
actually stalled WebView2 cleanup.

The opt-in native regression queues Sign out and Authorize before one STA wake,
then observes both the actual cleanup completion and authorization dispatch.
It repeats on the same host. Both fixed cycles produce
`cleared, authorize, cleared, authorize` and complete in 0.44 seconds locally.
The fixture calls the native loop directly, with an inert self-check page, empty
developer credentials and no user grant. It bypasses the credential service,
requires isolated `LOCALAPPDATA` matching its fresh empty test directory, and
sends shutdown/joins its host even when the observation fails. No Apple page,
authorization popup, real sign-out, credential deletion or account write occurs.

The existing `test-profile-clear.ps1` wrapper now runs both the cookie-removal
and ordering checks, each in its own empty profile. It restores its environment
and removes only its owned fixture directory afterward. Windows CI uses that
same wrapper. Runtime dependencies, settings, storage formats and network
destinations do not change. No interface redesign is involved.

This proves native command ordering and synthetic cookie cleanup, not actual
Apple authorization, SDK unauthorize completion, account metadata clearing,
credential expiry/revocation or fresh-PC sign-in. Those remain [release gates](release-readiness.md).

All local Windows contribution checks pass: formatting, strict default/demo
all-target Clippy, all-target tests (1,038/1,084 library tests, eight opt-in checks
excluded per ordinary suite), doctests, strict demo Rustdoc, demo build, gettext
and Node bridge/token checks. Standalone strict Clippy and 27 ordinary tests pass;
both opt-in profile fixtures and the native memory-policy fixture pass separately.
The separate restored-account read-only check passes in 6.776 seconds with
matching embedded page/bridge assets. It makes no playback, sign-out, popup or
account-write request. Source, binary and log hashes are in
[the qualification data](profile-order-check.json). Hosted all-feature/platform,
site and Nix checks remain required on the final feature head.
