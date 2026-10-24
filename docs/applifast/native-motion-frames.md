# Native playing-indicator frame evidence

Ordinary `--demo-shot` captures deliberately freeze animation. The demo build
now accepts `--demo-shot-frames 2` through `120` to capture a bounded native
sequence with animation enabled. It writes `NAME-000.png` onward and `NAME.json`
only when the sequence finishes. The default remains one frozen `NAME.png`,
without a manifest. This option is absent from ordinary builds.

Use a fresh output directory so an interrupted run cannot leave an older
manifest mistaken for its result. For example, after creating `.cache/motion`:

```powershell
cargo run --locked --features demo -- --demo --demo-page liked --demo-show queue --demo-data .cache/motion/profile --demo-size 1100x760 --demo-shot .cache/motion/capture.png --demo-shot-delay 2500 --demo-shot-frames 8 --demo-drag 10,740:10,740
```

Add `light` and/or `reduce-motion` to `--demo-show`, and repeat at `760x760`.
The demo uses isolated state, disables authorization restoration, media controls
and tray ownership, and sends no Apple account requests or audio. Public sample
artwork may load from Picsum as in existing demos. Its cursor is scripted through
the app's existing demo input hook. Normal application behavior is unchanged.

Each request carries its own index/time marker. The driver accepts only matching
readbacks from the root viewport; unrelated theme-transition images, stale
requests and repeated completed frames cannot become evidence. It waits 50 ms
after each readback/file write before requesting the next frame. The JSON records
the actual egui request times and dimensions, including QA overhead. It is not a
frame-rate, memory or CPU benchmark. Tests cover bounds, required paths, readback
ownership, sequencing, completion metadata, unchanged single-shot filenames and
write errors.

The [native evidence viewer](review-native-motion/index.html) contains eight
Windows cases, with eight actual full-window readbacks each. The report retains
their hashes, request times, source/binary hashes and the indicator crop boxes.
Each crop contains the red playing bars, checked independently of artwork loading:
normal `(294,428)-(314,447)` and narrow `(246,478)-(266,497)`, in physical pixels.
Moving cases have eight distinct indicator frames each. Reduced cases have one
identical indicator state across all eight frames. Lossless WebP clips contain
only decoded pixels from those native crops, with rounded request intervals and
no interpolation; identical adjacent frames may merge. A full first-frame still
provides context. The viewer starts paused and has an explicit playback control.
Chrome 155.0.8059.85 passes all sixteen viewer states. Six temporal browser
screenshots per moving case are distinct; three per reduced case are identical.
Both native-size context images and controls load correctly, with no script or
resource errors during that check. This checks actual rendered clip changes,
not merely image decoding. See the [browser result](review-native-motion/browser-check.json).

The base is `1078f85`; captures came from the candidate's changed demo entrypoint,
not an unchanged base executable. The accompanying report identifies those exact
dirty-during-capture source and binary hashes. No production UI, engine,
credentials, settings format, dependency or network destination changed.

Formatting, strict default/demo Clippy, default/demo all-target tests (1,040/1,088
library passes, eight opt-in checks excluded in each suite, plus 15 demo launcher
tests), doctests, strict demo Rustdoc, demo build, gettext and bridge/token checks
pass on Windows. A sandbox MSYS startup failure required rerunning gettext outside
the sandbox. Compiler cache hard-link warnings fell back to copying; no code lint
was disabled. Hosted platform/all-feature/site/Nix checks remain separate.

This closes only native playing-indicator motion and its Reduce motion state.
The separate [hover sequence](native-hover-frames.md) adds native cover-lift and
seek-hover evidence. The [full ledger](motion-acceptance.md) retains all eleven
requirements and each remaining acceptance check.
It does not establish all eleven PR #13 interactions, live Apple playback or
controls, lyrics follow behavior, other-platform runtime behavior, the deferred
resource sample or the gaming soak. Those [release gates](release-readiness.md)
remain open. The default-feature `1078f85` installer remains the latest functional
preview; the new capture flag is development evidence tooling.
