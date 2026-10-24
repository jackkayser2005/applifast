# Native cover, heart and page transitions

The [native viewer](review-native-transitions/index.html) replays 512 actual
Windows egui readbacks, covering four scenarios in both theme settings, two
window sizes and with Reduce motion on and off. Source is `8baea73`.

The cover scenario switches between two valid local SVG images through the
production crossfade path. The heart scenarios start the existing pulse on an
already-filled favorite in the player bar, track row and classic mini-player.
The page scenario opens Songs from Home through the existing action path.
No account write or playback command is sent.

Moving covers and pages have three or four distinct sampled pixel states,
including intermediate frames. Moving hearts increase their red pixel area,
remain visible with margins inside the detail crops and return to exactly their
starting pixels. Each case settles for at least its last five frames. With
Reduce motion, covers and pages have just their initial and immediate final
states; all sixteen heart frames are identical.

The temporary fixture records the capture index, supplies local covers, pauses
the demo and removes artwork URLs. It requests no window activation and drops
real keyboard/text input. All four production source files were restored after
the capture build. The retained zero-context fixture patch reproduces the exact
captured source hashes in an isolated Git index. Apply it to `8baea73` with
`git apply --unidiff-zero` only in a disposable capture checkout.

The stock 50 ms post-readback delay and production animation durations stay
unchanged. Actual request times include readback and PNG-writing overhead;
these samples establish native rendering, not frame rate or resource use.
The viewer retains direct lossless crops, deduplicates identical frames and
replays their actual timing without generating intermediate images. The
Transition / peak control selects a captured blend or the largest heart.

Every pixel outside the cover or main-player heart crops stays unchanged.
Classic mini-player checks exclude its separate paused time display, whose
intentional blink is visible in the full captures. That clock still blinks with
Reduce motion and remains a follow-up, not a passed accessibility check. The
classic skin keeps its own colors under both main-window theme settings.
Page navigation also updates the sidebar and top bar, so outside-page stability
is not asserted for that scenario.

The [evidence report](review-native-transitions/evidence.json) retains all
full-frame hashes, geometry, selected-pixel hashes, source/binary hashes,
measurements and fixture limits. All 120 browser selections match the native
decoded pixel hashes and rendered screenshot state counts. Forty replay checks
pause, resume and stop at their final frame. External requests are blocked and
no page script error occurs. Results are retained in
[browser-check.json](review-native-transitions/browser-check.json).

Natural clicks, real image arrival during listening, new favorite writes,
Settings interaction and the remaining queue/tint/lyrics/full-screen checks are
still required. See the [eleven-item ledger](motion-acceptance.md) and the
[release checklist](release-readiness.md). The `b9272ca` installer remains the
latest functional private preview; this evidence changes no application code.
