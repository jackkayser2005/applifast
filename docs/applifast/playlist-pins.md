# Local Apple playlist pins

A loaded Apple library playlist's More menu now offers **Pin to top** or **Unpin**.
It reuses the existing sidebar pin menu and `ArrangeLibrary` action. The sidebar's
right-click and drag behavior remains available. Pinned playlists stay above
unpinned rows under name, recent and library ordering. Dragging within the pin
block changes its local order.
Use the sidebar's Playlists filter to see the pinned playlist block.

Pins use the existing `pinned_contexts` preference in the Applifast settings file.
Settings reload preserves the exact typed playlist URIs and order. No migration,
new cache file, dependency, token storage or network request is added. Pinning
does not edit playlist contents or synchronize a preference to Apple's apps.
Library refresh and normal sorting do not overwrite the saved pins.

The page menu offers this action only for a playlist present in the loaded
library under the exact URI. A catalog playlist with a similar ID is not treated
as the library playlist. An arbitrary catalog-only playlist is not added to the
sidebar by this change. Existing sidebar pin behavior for other collection
types is unchanged.

## Verification

The input-driven regression first fails because the playlist page's More menu
has no pin item. With the change it verifies pointer clicks on Pin and Unpin,
one instance of each menu action, narrow/normal layouts, optimistic settings
updates, preservation of another pin and settings save/reload after each action.
It also verifies that a catalog URI cannot stand in for the library URI.
A second regression checks Apple pin order under different sorts, dragging a
pinned row, reversed refreshed library data and settings reload without changing
the underlying playlist objects or creating a local playlist-order override.

The [native HTML comparison](review-playlist-pins/index.html) uses the existing
fake created-playlist fixture and More-menu flag in both builds. Matching Windows
captures cover light/dark, normal/narrow and pinned/unpinned states. The baseline
sidebar already supports pins; the new page-menu command is the visible change.
The fixtures do not create or edit a real Apple playlist.
All 16 native captures were inspected, with matching dimensions and the new
menu item visible in each candidate state. The HTML's 16 selector paths and
Before/After controls pass a static Node VM check.

Browser rendering remains unchecked. On October 14 the allowed browser workflow
could not connect because its privileged pipe bridge was unavailable and the
client was not trusted. No alternative browser automation was used. Native
captures, input-driven menu checks and static selectors are separate evidence.
Fresh-PC Apple authorization and the complete public-release checklist remain open.

Windows formatting, strict default/demo all-target Clippy, all-target tests,
doctests, strict demo Rustdoc, demo build, gettext consistency and Node bridge/
token-signer checks pass. Library suites pass 1,026 default and 1,071 demo tests,
with six opt-in checks ignored in each ordinary run. Translation updates change
generated references only. Candidate hosted CI and optimized package results
are recorded on the feature PR; the local checks do not prove other-platform
Apple playback or fresh-PC authorization.
