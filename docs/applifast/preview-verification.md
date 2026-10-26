# Current preview verification

This follow-up records a real installed-preview diagnostic and actual browser
checks of the existing visual comparisons. It does not declare the Windows
beta, gaming budget or interactive motion accepted.

## Installed memory preview

The running Windows x64 app matches clean source
`3508c615f419e7551484458bb1cff51f5538c795` and executable SHA256
`2ffa8182f937e0ae12baa5e82def2c64532c5b0bb45337070eee5e6335d1ecc5`.
Its read-only status client reports paused before measurement. Preflight finds
an unminimized application window, so the requested minimized acceptance run
has not started. The visibility check also identifies Winit's 16-by-16 event
target window; future preflight excludes that utility surface.

The existing process sampler instead takes a short diagnostic, with zero
warm-up and 30.958 seconds of observation. All 31 snapshots pass independent
checks that per-process private bytes sum to the aggregate and detailed PID
lists match their inventories. The peak contains the app and six WebView2
helpers. Only counters and process identities are retained locally; no complete
command lines, authorization responses, song metadata or library data are
published.

| Diagnostic counter | Result |
| --- | ---: |
| Minimum aggregate private memory | 508.531 MiB |
| Maximum aggregate private memory | 514.688 MiB |
| Native app at the aggregate peak | 362.227 MiB |
| All helpers at that same peak | 152.461 MiB |
| First / last aggregate sample | 513.520 / 513.844 MiB |
| Average total CPU, normalized across logical processors | 0.044162% |

Private bytes, resident working set and GPU memory are different measures.
These values are not directly interchangeable with Task Manager's default
Memory column. The short sample cannot qualify either paused CPU or the
250 MiB limit. Its `AccountingComplete` is false with only
`acceptance_duration_not_met`; no process-counter mismatch was found. The raw
report is kept locally at `.cache/memory-preview-3508c61-visible-diagnostic.json`,
SHA256 `6a5453a982fbbf866483dd25f02153cd07e1568c121e02349c86be561bbad671`.

Playback state is checked before this diagnostic, not continuously. Visible
windows, normal PC use, browsing history and playback-host initialization make
it unsuitable for causal comparison with earlier builds or the short offline
font demo. It establishes that native memory remains a substantial investigation
target. Neither the on-disk artwork inventory nor the process totals identify
the current native allocation owners. Cache limits have not been reduced on
the basis of this sample.

## Rendered comparison pages

An isolated Microsoft Edge 155.0.4283.62 session renders the public sample
comparisons from application source
`50024ce10d12de39f9d589c5980155b98cde4588`. The Codex in-app browser connection
remains unavailable. The separate test browser uses no signed-in profile and
no Apple account. A temporary server exposes only `docs/applifast` on loopback;
the browser and server are closed after verification.

| Comparison | Actual browser states checked |
| --- | ---: |
| Overall integration: Songs, queue, Settings, fixed-size mini-player | 28 |
| File-backed fallback fonts | 8 |
| Apple sign-in disclaimer | 8 |

Each state selects the displayed controls, checks the pressed Before/After
button, decodes the selected native PNG and verifies visible, nonzero image
dimensions. Mini-player sizing is correctly disabled, so its duplicate narrow
captures do not count as extra interactive states. Index Previous/Next,
last-item disabling and direct-link selection also pass. The only recorded
console error is a missing optional `/favicon.ico`.

The [machine-readable checks](preview-verification/browser-checks.json) record
the selected images and dimensions. Representative full-page browser captures
were visually inspected: [integration](preview-verification/integration-browser.jpeg),
[font memory](preview-verification/font-memory-browser.jpeg) and
[sign-in footer](preview-verification/apple-footer-browser.jpeg). The index uses
an internally scrollable comparison frame; these browser images supplement the
original native PNG pairs rather than replacing them.

This closes the previously missing browser-rendering evidence for these three
comparisons. It does not validate every individual feature gallery, native
animation timing, actual interactive media controls, fresh-PC authorization,
audible listening, Windows installer wizard appearance or the gaming session.

## Hosted checks

All seven required jobs pass on each exact feature source:

- Font memory `37b60b3`: [run 39330511147](https://github.com/jackkayser2005/applifast/actions/runs/39330511147).
- Packaged notices `3508c61`: [run 39331872823](https://github.com/jackkayser2005/applifast/actions/runs/39331872823).
- Apple footer `50024ce`: [run 39336048436](https://github.com/jackkayser2005/applifast/actions/runs/39336048436).

These include quality, documentation, Windows x64/ARM, Linux, macOS and the full
Nix build. Other-platform test success does not establish Apple playback there.
Required checks must pass again on the final integrated release source.
