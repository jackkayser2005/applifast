# Native browser cookie cleanup

The opt-in `native_profile_cookie_clear_round_trip` fixture uses actual Windows
WebView2 APIs on an STA with the existing bounded message pump. Its wrapper gives
it a fresh, empty disposable directory. It rejects missing, relative or nonempty
profile paths before WebView2 initialization and never reads Credential Manager.
The native parent stays hidden. No page navigation, Apple SDK, authorization
popup, account request or audio is started.

The fixture adds synthetic Secure/HttpOnly session and persistent cookies on two
reserved `.invalid` domains, then reads them through the native cookie manager.
Disk-cache-only cleanup preserves both. Complete profile cleanup removes both.
Adding a fresh cookie succeeds afterward, and repeated complete cleanup removes
that cookie too. The clear completion uses the same reporting helper as the
production sign-out handler. The wrapper removes its profile afterward using
validated paths and reparse-point checks, including when the test fails.

| Native snapshot | Cookies |
| --- | ---: |
| Fresh profile | 0 |
| Synthetic session and persistent cookies added | 2 |
| Disk-cache-only cleanup completed | 2 |
| Complete profile cleanup completed | 0 |
| Fresh cookie added after cleanup | 1 |
| Repeated complete cleanup completed | 0 |

The Windows x64 check passes on Evergreen `155.0.4283.62` in 0.44 seconds. The
[qualification data](profile-cookie-check.json) identifies the tested source and
binary. Windows CI runs the same wrapper explicitly; ordinary host tests keep
the runtime-dependent fixture ignored. No dependency or runtime behavior changes.

Microsoft documents the [cookie manager's profile scope](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2cookiemanager)
and [asynchronous profile cleanup](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2profile2).
This fixture waits for completion before each dependent operation. It does not
verify the application's immediate sign-out/reauthorization ordering, actual
Apple cookies, local storage, account metadata, revoked/expired credentials or
fresh-PC sign-in. Those remain [release gates](release-readiness.md). The
[synthetic cleanup-error regression](profile-clear-errors.md) separately checks
failure reporting, without establishing native cookie removal.

Local Windows checks pass: formatting, strict default/demo all-target Clippy,
all-target tests (1,038/1,084 library tests, eight opt-in checks excluded per
ordinary suite), doctests, strict demo Rustdoc, demo build, gettext and Node
bridge/token checks. Standalone strict Clippy and 27 ordinary tests pass, with
the runtime-dependent cookie check run explicitly. Missing/relative/nonempty
profile-path rejection also passes, preserving the nonempty fixture's marker.
Hosted all-feature/platform, site and Nix results remain separate gates.
