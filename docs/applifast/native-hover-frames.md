# Native cover and seek hover frames

The [native viewer](review-native-hover/index.html) replays 128 actual Windows
egui readbacks across light/dark, normal/narrow and Reduce motion cases. It
checks two PR #13 rendering paths: the cover's hover lift and Play affordance,
and the seek bar's hover color, thickness and knob.

Source is `69b5b61`. A shared temporary fixture delays existing forced hover
until capture index one, pauses the demo and removes artwork URLs. The stock
50 ms post-readback delay and all production animation durations stay unchanged.
The fixture patch, exact source/binary hashes and all full-frame hashes are
retained. Production source bytes are restored after building the capture tool.
There is no account, audio, tray ownership or normal-player mutation.

Each moving case contains four distinct card/seek crops, with intermediate
edge/color pixels. The card edge strip has three native states. The cover rises
four physical pixels. The seek-track solid interior grows two rows: four to six
at normal width and three to five at narrow width, whose edges are antialiased.
The knob appears above the track; it is absent before hover. A paused position
keeps ordinary progress updates from being mistaken for hover motion.

Each Reduce motion case has exactly two states: initial and immediate final.
Every frame after the first is identical in both detail crops. All cases settle
for at least the final five frames, and every pixel outside the two crops stays
unchanged. Real request intervals are preserved in the viewer; capture and file
writing overhead make them unsuitable as a frame-rate or performance benchmark.
Repeated crops share one lossless PNG. No tweening or generated frames are used.

The separate browser check passes 48 rendered before/intermediate/after
selections and sixteen pause/resume/end replays. Actual browser image pixels
retain three distinct selected states for moving cases and two for Reduce
motion. All external requests are blocked; no page script error occurs.
See [browser results](review-native-hover/browser-check.json).

This verifies native rendering of those two paths. Existing deterministic
pointer/action regressions remain separate, as do natural pointer acceptance,
live Apple controls, all other motion paths and the original release gates.
See the [eleven-item ledger](motion-acceptance.md). The `b9272ca` private installer
remains the latest functional preview; this evidence adds no application feature.
