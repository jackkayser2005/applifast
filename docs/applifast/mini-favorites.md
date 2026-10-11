# Mini-player Apple favorites

This draft follows PR #44 (`e536995`). The expanded mini-player replaces the
inactive Apple EQ slot with the current song's heart. Its title-bar right-click
menu also offers Add to Favorites, including when rolled up. Filled hearts and
the menu's Favorites label are read-only. Song, transport, volume, queue, scale,
skin dimensions, window ownership and Spotify EQ behavior are unchanged.

Both entry points emit the existing ToggleSaved action after drawing. The
shared Apple state preserves typed identity, applies immediately, confirms exact
resources, rolls back unconfirmed results, protects against old reads/sessions
and excludes pending flags from confirmed disk state. No new network calls,
dependencies, settings, storage formats or credentials are introduced.

Focused deterministic regressions exercise accessible heart activation at all
four skin scales, read-only pending hearts, failed-write rollback and the actual
rolled-up title-bar menu's current-song action. No live account is used.

Windows formatting, strict default/demo Clippy, 1,016 default and 1,053 demo
library tests (five opt-in tests ignored each), the remaining binary/integration
tests, doctests, strict Rustdoc, demo build, gettext and bridge/token-signer tests
pass. Hosted validation of this follow-up remains a separate gate.

[Native visual comparisons](review-mini-favorites/index.html) contain 36 inspected
Windows captures: matching ready, pending, confirmed and error states in both
themes at 1x/2x, plus four candidate-only open-menu captures. Selector, file-path
and PNG-dimension checks pass. The baseline has no deterministic options-menu
fixture, so no Before menu image is claimed. The rolled-up menu is pointer-tested
but not pictured; HTML browser rendering remains unchecked. The classic skin
supplies its own colors, independent of the app's light/dark theme.
Real-account favorite additions and propagation, restart acceptance, custom-skin
visual acceptance and gaming measurements remain pending. Removal still requires
Apple Music. This is not a public release or complete favorites parity.
