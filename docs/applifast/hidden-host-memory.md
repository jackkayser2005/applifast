# Hidden playback host memory target

The independent Windows playback host keeps its WebView renderer logically
visible even though its native parent window is hidden. That preserves the
MusicKit startup behavior required after a previous fully hidden cold start
stalled. The old visibility helper also selected the normal memory target
whenever the renderer was active, including this background host.

The helper now reads the controller's actual parent-window visibility before
selecting the target. The hidden host requests Low while keeping its renderer
active. A visible diagnostic window retains Normal, and a hidden/minimized
diagnostic view retains Low. The renderer visibility decisions, authorization
popup handling, STA/message pump, playback commands and native window ownership
are unchanged. No suspend/stop call, browser argument, dependency, credential,
storage format or network destination is added.

Microsoft documents that [the memory target can be set independently of
controller visibility](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_19?view=webview2-1.0.4129.50).
Scripts continue running, but this is a best-effort request. It can page memory
out and affect subsequent access time. A lower working set does not establish
lower private commitment. This change is not a measured resource-budget or
latency improvement, and it needs playback/seek responsiveness acceptance.

## Native regression and SDK checks

The existing `--self-check` now exercises the embedded host's relevant
combination against real WebView2: active renderer and hidden native parent.
It reads both native visibility and the actual memory-target property. Before
the fix it reports `rendererVisible: true`, `parentVisible: false`,
`targetLevel: 0` (Normal), then fails. With the fix the same check reports
`targetLevel: 1` (Low) and exits successfully. The fixture also restores the
original controller visibility afterward.

Two actual Evergreen `155.0.4283.45` starts pass, including immediate reuse of
an initially empty owned profile, in 0.314 and 0.265 seconds. These are setup
durations, not playback latency. Self-check reads no Apple credentials and
performs no Apple navigation, sign-in, audio or account write.

The new opt-in native test
`windows::tests::embedded_host_reads_library_and_catalog_without_playback`
uses the same embedded `attach` path as the application. Cold SDK startup,
restored authorization, 100 library songs and 10 catalog results pass without
playback, authorization popup, sign-out or account writes. Its profile must be a
fresh absolute directory explicitly supplied in both `LOCALAPPDATA` and
`APPLIFAST_NATIVE_READ_PROFILE`. It rejects missing setup before native startup
and refuses an existing Applifast profile. The test does not set process-wide
environment variables internally. EOF shuts down its owning STA even when an
assertion fails. This account-dependent check stays ignored in ordinary suites.

The standalone host passes formatting, strict all-target Clippy and 21 ordinary
tests, with two account/platform checks ignored. The new read-only account test
passes separately with its explicit profile guard. The installed player is left
alone. Windows main-app formatting, strict default/demo all-target Clippy,
all-target tests, doctests, strict demo Rustdoc, demo build, gettext and Node
bridge/signer checks pass. Library suites pass 1,027/1,073 with six opt-in checks
ignored per ordinary suite. Git Bash required named-object access outside the
sandbox for gettext. Hosted-platform and final package results belong to the
exact feature head; they do not follow from these local checks.

## Remaining acceptance

The [installed integration baseline](integrated-performance.md#october-15-latest-installed-integration-preview)
still reaches 503.637 MiB aggregate private memory. It predates this policy and
does not measure its effect. The original 250 MiB limit remains failed in
existing observations. No comparison with an empty diagnostic host can replace
the integrated app, protected playback, minimized-state samples or gaming soak.

Before integration, check real playback, complete tracks, seeking, mixed queues,
authorization popup/retry, first-song and switching latency, then repeat the
controlled process-group measurements. Diagnostic show/hide/minimize interaction
also remains unverified; the native self-check covers the hidden active-renderer
state. The independent read-only checks do not close those gates. There is no
visible application layout change or release claim. The previously verified
`1742628` private installer remains the current
download until a replacement is qualified.
