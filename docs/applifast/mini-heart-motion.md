# Mini-player favorite motion

The Apple mini-player painted its heart at a fixed size. Its favorite button and
options menu did not start the shared pop either. The main-player/row follow-ups
therefore left the mini-player without the same feedback, even from B.

The existing mini-player button and menu now start the URI-scoped heart pop.
The painted hollow/filled icon consumes the shared scale, bounded by its fixed
skin cell height. Resting size, skin layout, hit area, accessibility and filled
read-only semantics stay the same. Reduce motion is already applied by the
mini-player and stills or cancels the effect. No new action, setting, storage
format, dependency, credential handling or network access is introduced.

The timed actual-interface regression fails before the fix on the unchanged
9-pixel heart. Afterward it passes 48 cases: both palettes, all four skin scales,
button/menu/B input and normal/reduced motion. It inspects the painted image
bounds inside the unchanged accessibility cell, verifies enlargement and settling,
Reduce motion interruption/no revival, and failed-confirmation rollback to the
add button. The menu needs its normal popup layout pass before activation.
All fixtures use isolated sample data with an offline backend and no account
writes or audio.

The [native comparison](review-mini-heart-motion/index.html) uses matching
ready/pending states in both themes at 1x and 2x, with the built-in skin and
the same pointer. Before application code is identical to `89eba05`; its demo
binary was built at `6357c67` with only later docs/CI differences. Capture
provenance and pixel comparisons are retained with the gallery.
All eight Before/After pairs have identical decoded pixels at rest. Chrome
155.0.8059.85 also loads all sixteen viewer selections with the expected image
dimensions and no script/resource errors in the serialized check, which waits
for each selected image to decode. Its normal/narrow viewer captures were
inspected. See [native provenance](review-mini-heart-motion/evidence.json) and
[browser results](review-mini-heart-motion/browser-check.json).

Windows formatting, strict default/demo all-target Clippy, default/demo all-target
tests (1,041/1,090 library passes, nine opt-in tests ignored in each ordinary
suite), doctests, strict demo Rustdoc, the demo build, gettext and Node bridge/token
checks pass. Translation changes update references and extraction dates only.
Hosted platform/all-feature, documentation-site and Nix checks remain separate.

These frozen images check resting appearance and dimensions, not animation.
The regression uses timed egui events and painted shapes, not Windows keyboard
injection. Real native interaction/timing, custom skins, new-favorite Apple
confirmation and resources remain separate [release gates](release-readiness.md).
The existing `8e89904` private installer predates this follow-up.
