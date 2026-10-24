# Reduce motion across shared controls

Four remaining controls bypassed the shared motion policy: settings switches,
the player-bar artwork tint, collection drag shifts and sidebar drag shifts.
They now use the existing helpers. Reduce motion snaps them to their targets,
including an ongoing tint fade, and keeps the internal animation state in step
when motion is enabled again. Switches use the shared eased boolean transition
at the existing style duration when motion is enabled.

This follow-up to PR #13 changes no layout, controls, preferences, storage,
credentials, dependencies or network access. Drag targets and drop actions
retain their existing behavior. The setting already reaches both the main and
mini-player contexts.

Two regressions failed before the fix and pass afterward. One clicks the actual
shared switch, examines its painted knob in both themes, enables Reduce motion
during the transition, clicks it off and re-enables motion. The other interrupts
an actual player-bar tint fade in both themes, checks immediate target colors,
clears the tint and verifies the next ordinary fade starts from that color.
Existing shared motion and drag tests cover the reused paths separately.

[Native comparisons](review-reduced-motion-controls/index.html) match Windows
Settings at normal/narrow sizes and in both themes. Screenshot mode holds
animations at their final state; these resting views check that the shell is
preserved and cannot demonstrate transition timing. Input-driven egui tests
provide the transition regression evidence. Interactive desktop timing and
rendering the comparison HTML remain separate checks.

Windows formatting, strict default/demo all-target Clippy, 1,019 default and
1,057 demo library tests, the remaining binary/integration targets, doctests,
strict Rustdoc, demo build, gettext and bridge/token-signer self-checks pass.
Six opt-in account/credential checks were ignored in each ordinary library
suite; this interface-only change does not rerun live account writes or sign-in.
All eight native captures were inspected. Each before/after resting pair is
pixel-identical; dimensions and selector paths pass. Translation updates change
only extraction dates and source references, with all translated text unchanged.

The Flatpak release-metainfo regressions pass locally. Linux launcher fixtures
cannot run here without Ruby and POSIX tools; the release-name symlink fixture
fails with Windows privilege error 1314. Their hosted quality job, all-feature
checks, documentation-site build, Nix and other-platform tests remain CI gates.
The earlier sandbox temporary-directory failure was resolved by placing Python
temporary files in the workspace; these remaining platform constraints were
not suppressed or changed into passing results.
