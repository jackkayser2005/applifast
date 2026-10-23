# Player-bar artwork failure recovery

When a song changes, the previous cover stays visible while the new image is
pending. A failed load previously counted as still pending, so another song's
cover could remain indefinitely. Both successful loads and errors now finish
that wait. The existing 250 ms transition shows the new image or the existing
music-note placeholder. Reduce motion switches immediately.

The paint regression uses the real egui image loaders, a valid in-memory PNG
and corrupt PNG bytes. It waits for asynchronous decoding before asserting the
error, then inspects the actual tessellated texture IDs. The unchanged production
code fails because the old texture is still painted after 500 ms. The fix passes
in light/dark themes with Reduce motion both on and off. It also confirms that
forgetting the failed image and supplying valid bytes paints the recovered
texture. Existing pending and rapid-skip state regressions remain intact.

Windows formatting, strict default/demo all-target Clippy, default/demo
all-target tests (1,043/1,092 library passes, ten opt-in checks excluded per
ordinary suite), demo doctests and strict demo Rustdoc pass. The first regression
invocation used a short name with `--exact` and selected zero tests; only the
subsequent fully qualified failing run is reproduction evidence. An earlier
fixture incorrectly expected immediate PNG decoding. That fixture failure was
not evidence of the product defect.

This changes no playback command, queue occurrence, authorization, storage,
dependency, image source or network destination. The normal listening app is
untouched. The [HTML comparison](review-cover-recovery/index.html) contains
sixteen native before/after captures in both themes, sizes and Reduce motion
settings. All eight pairs differ only inside the 56-by-56 player-bar cover;
every other pixel matches. Both builds use a shared temporary fixture that
seeds an old SVG, supplies corrupt PNG bytes and disables other demo artwork.
Exact source/binary hashes and patches are retained. Production source is
restored afterward. These frozen end states do not prove live transition timing.
The separate local Chrome viewer check passes all sixteen selections, PNG
dimensions and loading the selected comparison through the shared review index.
External browser requests are blocked. See [browser results](review-cover-recovery/browser-check.json).

Required hosted all-feature/platform checks and an updated downloadable preview
remain to be qualified for this follow-up. It does not close the other PR #13
interactions or release gates.
