# Native queue arrival frames

The [native viewer](review-native-queue/index.html) replays 256 actual Windows
egui readbacks for the compact queue side panel and full Queue page. It covers
light/dark, normal/narrow and Reduce motion cases using source `99e0132`.

The offline paused fixture uses existing `QueueMany` actions. It seeds two
manual entries, then adds two more copies of the first song. Every recorded
model and UI snapshot retains both additions before the unchanged context
tracks. The manual count changes from two to four, with three copies of the
first song in that prefix. It does not connect a real account or playback host.

Both newly added rows fade and rise. Their captured cover edges rise one native
pixel at normal width and two at narrow width; earlier motion is not captured.
Cover contrast increases toward its final value. Moving cases contain two or
three post-addition pixel states; Reduce motion has just the immediate final
state. The existing two-row manual prefix is pixel-identical in every frame,
and the new rows settle for at least their last five frames.

Before addition, the new-row crop contains the following context section.
Adding manual entries moves that section down. Compare First added frame with
Settled to inspect animation rather than attribute all structural change to it.
New toasts and context movement outside the detail crops are expected, so
global pixel stability is not asserted.

A four-second startup delay lets the ordinary add debounce expire. The first
1.8-second pilot hit that guard and did not add; this was a fixture failure,
not a queue defect. The stock 50 ms post-readback capture delay did not resolve
normal-width rise. The final temporary capture tool uses 5 ms instead, without
changing production animation duration, easing or opacity. Actual request
intervals include readback and PNG-writing overhead. These are sampled native
states, not a frame-rate, resource or click-latency benchmark.

The temporary fixture pauses sample playback, removes artwork URLs, requests
no activation and drops real keyboard/text events. All production source bytes
are restored after building it. Source/binary/full-frame hashes, geometry,
before/after queue states and per-frame manual counts are retained in the
[evidence report](review-native-queue/evidence.json). The zero-context fixture
patch reproduces all captured source hashes in an isolated Git index. Apply it
to `99e0132` with `git apply --unidiff-zero` in a disposable capture checkout.

All 96 browser selections match the native decoded pixels and rendered
screenshot state counts. Thirty-two pause/resume/end replays pass with external
requests blocked and no page script errors. Results are retained in
[browser-check.json](review-native-queue/browser-check.json). Identical lossless
crops share files; the viewer generates no intermediate frames.

This adds no application behavior. Natural queue gestures, live Apple
playback, stale responses, restart restoration and the remaining
[motion](motion-acceptance.md) and [release](release-readiness.md) gates remain
separate. The `59062de` private installer remains the current functional preview.
