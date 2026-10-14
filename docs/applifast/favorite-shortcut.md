# Apple Favorites keyboard shortcut

This follow-up to PR #45 (`7489d72`) enables **B** for the playing song in
the main and mini-player windows. The keyboard-shortcut dialog lists it as
Add to Favorites. It does nothing for an existing or pending favorite, while
typing, while a dialog is open, for an episode, or without a current song.
Spotify mode retains its existing like/unlike binding.

The shortcut emits the existing add-only SetSavedMany action with the exact
playing URI. The shared Apple path applies the heart immediately, confirms
the exact resource, rolls back unconfirmed results, rejects stale events and
excludes pending flags from the confirmed restart cache. Removing favorites
still requires Apple Music. No new request type, dependency, storage format,
credential, preference or network destination is introduced.

The focused regression checks library identity, false/unknown metadata,
existing and pending favorites, retry after failed confirmation, text and
dialog guards, episodes and no current song. Existing tests cover shared
optimism, partial confirmation, stale reads, account changes and persistence.

Windows formatting, strict default/demo all-target Clippy, 1,016 default and
1,054 demo library tests (five opt-in tests ignored each), remaining
binary/integration tests, doctests, strict Rustdoc, demo build, gettext and
bridge/token-signer tests pass. A separate native credential-store round trip
passes using temporary dummy grants. Hosted all-feature and other-platform
checks remain separate gates.

[Native visual comparisons](review-favorite-shortcut/index.html) match the
open shortcuts dialog in light/dark themes at normal/narrow sizes. The only
visible change is the additional B row within the existing scrollable dialog.
All eight native captures were inspected; selector paths and PNG dimensions
pass. Each pair uses the same prewarmed demo artwork cache.
These captures and keyboard tests do not establish real-account writes,
fresh-account authorization, browser rendering of the HTML or gaming acceptance.
