# Cancelling Apple authorization

Sign-out used to wait for the transport command chain, including an unresolved
`MusicKit.authorize()` promise. Closing an authorization window could leave that
promise pending, preventing cancellation from completing and blocking a retry.
The new regression reproduced this before the fix.

The bridge now races authorization against an explicit cancellation signal.
Sign-out cancels the active attempt before waiting for the transport chain.
Closing the native authorization popup invokes the same cancellation path before
reporting the existing closed-window message. A new attempt has its own identity.

The bridge retains the accepted user grant only in memory alongside MusicKit.
A late SDK completion restores that accepted grant instead of publishing the
cancelled result or overwriting a newer sign-in. The attempt and session are
checked again immediately before publication. Sign-out clears the accepted grant,
including when it starts before MusicKit configuration finishes. Persistent
credentials still use Windows Credential Manager; no storage format changes.

The Node bridge checks cover unresolved authorization during sign-out, a retry
before the old result arrives, native-style popup cancellation, an old result
after a newer sign-in, cancellation between SDK resolution and publication, and
sign-out during configuration. They use synthetic tokens only. The original
unresolved-promise check failed before the fix; all bridge checks now pass.

Windows formatting, strict default/demo all-target Clippy, 1,017 default and
1,055 demo library tests plus remaining targets, doctests, strict Rustdoc,
demo build, gettext and bridge/signer checks pass. Six opt-in tests remain
ignored in each ordinary library suite. These deterministic checks do not prove
actual Apple popup behavior, native profile clearing, expiry/revocation or
fresh-PC sign-in. Those checks remain on the [release checklist](release-readiness.md).

The updated host also passed the separate muted real-account regression in
43.80 seconds: restored protected authorization, 100 library rows, shelves,
details/search, playback, seek, pause and actual saved-position resume with both
queue occurrences retained. It used an owned workspace profile because of the
recorded execution-environment asset-location issue. No popup, Sign out or
account-write command was sent; this does not establish cancellation acceptance.

This removes the wait for the old authorization promise. It does not impose a
deadline on MusicKit's pause or unauthorize operations. The popup, sign-in card,
credentials, account-cache clearing and network destinations retain their
existing behavior and layout.
