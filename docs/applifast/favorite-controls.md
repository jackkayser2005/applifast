# Add-only Apple favorites

This draft desktop slice follows the exact confirmation boundary in PR #43
(`01406e4`). Live-account favorite additions remain unverified.

Hollow hearts in song rows and the player bar, plus song and selection menus,
offer **Add to Favorites**. Filled hearts remain read-only because removal has
no verified route. There is no layout, navigation or settings change. A dedicated
mini-player favorite button remains pending; its shared song state stays in sync.

A selection adds up to 100 distinct non-favorite songs. Repeated identities and
already favorite or pending members are skipped. Library, catalog and playback
IDs remain separate. Validation finishes before any optimistic state changes.
The existing asynchronous write channel sends the command independently of
playback, without automatically retrying the mutation.

Hearts fill immediately, including in Favorites and the sidebar shelf. Exact
typed-resource confirmations retain successful members. False, unknown,
malformed, missing or duplicate results restore each unconfirmed member's prior
known or unknown state and show a fixed refresh-before-retry message. Refresh
before retrying: an interrupted write may have reached Apple.

Metadata reads issued before completion cannot undo the result. Later reads are
ordered per changed identity and can reflect external edits. Sign-out or host
replacement clears pending writes and local updates; old-session events cannot
affect another account. Pending hearts are excluded from confirmed disk state.
Confirmed favorite flags use the existing account snapshot, without changing
its format, paths, credentials or token storage. Playback queue occurrences and
ordering are unchanged.

This reuses the MusicKit POST and bounded confirmation GETs documented in
[favorites evidence](favorites-write-evidence.md). There are no new dependencies,
destinations, settings, telemetry or audio persistence.

Regressions cover immediate state, duplicate clicks, uploaded identity, partial
rollback, wrong/malformed/ambiguous resources, pending cache state, ordered reads,
authorization, selection bounds, old-session responses, accessible add buttons
and add-only selection menus. [Matching Windows captures](review-favorite-controls/index.html)
show Songs with a hovered row, plus ready, pending, confirmed and error states
with Favorites filtering enabled, in light/dark at normal/narrow sizes.
All 40 native captures were inspected. Selector, image-path and PNG-dimension
checks pass; browser rendering of the HTML remains unchecked because browser
automation could not load its runtime module. Baseline images show the unchanged prior state because it has no add
controls or favorite-write demo fixtures. Open menus and live motion are not
pictured. The screenshots contain deterministic demo data, never account data.

Windows validation passes: formatting, strict default/demo Clippy, 1,016 default
and 1,051 demo library tests (five opt-in tests ignored in each), the remaining
binary/integration tests, doctests, strict Rustdoc, demo and optimized release
builds, translation checks, and the bridge/token signer tests. Hosted validation
of this branch and real-account favorite writes remain separate gates.
The optimized binary also passes the dummy-token package checks and isolated
per-user installer, shortcut, upgrade and uninstall checks. These fixtures never
launch the app or authorize an Apple account.

Live-account additions, server propagation, genuinely uploaded songs, restart
acceptance, menu captures, mini-player editing and gaming measurements still need
verification. The earlier `9cb3a36` preview's confirmed audible catalog playback
does not verify these additions. No public release or full favorites parity is
claimed.
