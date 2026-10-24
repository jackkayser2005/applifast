# Paused mini-player clock and Reduce motion

With Reduce motion enabled, the classic mini-player now keeps its paused time
visible. Turning the setting off restores the usual blink. The skin, controls,
layout, playback state and account data are unchanged. Shaded mode already
draws a steady time and uses a separate drawing path.

The [native comparison](review-paused-clock/index.html) matches the `8baea73`
baseline contexts retained in PR #93 against candidate `37647f5`. It includes
both main-window theme settings, 550 × 232 and 275 × 116 skin sizes, and both
motion settings. The classic skin retains its own colors under either theme.

The regression renders the actual four digit sprites and sign at visible and
blank blink phases, then toggles Reduce motion on and off at both skin scales.
With the same test source, restoring the original clock condition reproduces
the failure: the reduced-motion blank phase selects blank sprite UVs. The fix
uses the shared motion setting to keep selecting the visible digits. This
checks the real drawing path rather than a duplicate blink predicate.

All 128 candidate native readbacks pass their scoped checks. Each Reduce motion
case has one identical full-frame state across sixteen frames; each ordinary
case has two blink states and retains the matching baseline blank state.
Every pixel outside the clock region is identical to its baseline. The
temporary fixture pauses the demo, removes artwork URLs, requests no activation
and drops real keyboard/text events. It changes no clock timing, layout or
capture cadence. All production source bytes are restored after building it.

Windows formatting, strict default/demo all-target Clippy, default/demo
all-target tests, doctests and strict Rustdoc pass. Library counts are
1,043 default and 1,093 demo passes, with ten opt-in checks ignored in each
ordinary suite. Required hosted all-feature/platform, docs and Nix checks
remain separate final-head gates.

Exact source/binary/full-frame hashes, sprite regression provenance and native
results are in [evidence.json](review-paused-clock/evidence.json). The retained
zero-context capture patch is checked against every captured source hash in an
isolated Git index. Apply it to `37647f5` with `git apply --unidiff-zero` only in
a disposable capture checkout. Browser results are retained in
[browser-check.json](review-paused-clock/browser-check.json).
All 24 browser selections match the native decoded pixels and rendered state
counts; eight pause/resume/end replays and keyboard frame selection pass with
external requests blocked and no page script errors. Ordinary-motion stills
can show different blink phases because their request times differ. Replay
confirms the original blink is retained.

The optimized `59062de` private installer and ZIP include this fix. Their
checksum, six-file payload, clean-source manifest, PE/default-help and isolated
dummy-token install/upgrade/uninstall checks pass. A credential-pattern audit
of 96 fetched published branch tips, 6,902 raw history blobs and this private ZIP
finds no matching secret paths or key/developer-JWT patterns outside the declared
app-token companion. Opaque/encoded payloads, image text and remote unreachable
history are outside that audit. The original preview source branch is retained.

The app and installer remain unsigned development previews. These checks make
no account write, audio command or resource claim. Natural Settings use, window
recreation, real listening on the exact executable and the broader
[release checklist](release-readiness.md) remain open.
