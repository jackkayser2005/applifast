# Readable Apple collections with a side panel open

The preceding toolbar fix kept Search and shortcuts accessible at the 760-point
window minimum, but the collection still reserved a 160-point cover, a fixed
220-point filter and enough row columns to leave no room for names.

Apple collection content below 360 points now uses a 64-point cover and a smaller
heading, places Filter below Play/Shuffle/More, and prioritizes the table's song
and artist names. Favorites and More remain visible/available. Row artwork and
duration values return when space permits. Playlist-order, title and duration
sorting remain available, including duration sorting while its values are hidden.
The compact header and standard rows show full titles on hover. Songs and Favorites use the existing
translated Library caption instead of calling themselves playlists.

The maintainer approved this responsive appearance on October 22, 2026:
"Approve both layouts" (#82 and #83). Implementation/release gates remain separate. The
[comparison](review-narrow-collections/index.html) provides matched native Windows
loaded/menu captures in both themes at 760×760 and 1100×760. The wider layout
retains its arrangement; its Library caption also changes.

The original song-menu demo flag opened both the library menu and the compact
current-song menu. Its corrected scope excludes compact rows for that flag;
the separate queue-menu flag is unchanged. The baseline is production UI from
`6fd6864`, with that same demo-only fixture correction applied. The capture helper
backs up the two owned source files, checks their hashes, builds/captures the
baseline, then restores the candidate byte-for-byte in `finally`. Baseline and
candidate source/binary hashes are retained locally. This is a qualified baseline,
not a claim that the unmodified `6fd6864` demo fixture produced those menus.

The regression checks actual painted glyphs and backgrounds, so hidden full text
in accessibility metadata cannot pass a clipped heading. The baseline fails on
the Songs heading. The same test uses real pointer events to play Tides, append it
from More and sort by duration, in both themes with Queue/Lyrics and narrow/normal
sizes. Each scenario uses a fresh app to respect the existing double-click queue
debounce. A separate failing assertion reproduces two forced menus before the
fixture correction and expects one afterward.

No backend, queue semantics, credentials, storage format, dependency or network
destination changes. The production layout changes are Apple-only. Native stills
and mocked local playback intent do not prove live motion, actual Apple playback
or writes, other-platform runtime behavior, or resource acceptance. Those remain
in the [release checklist](release-readiness.md).

Windows formatting, strict default/demo all-target Clippy, all-target tests
(1,040/1,088 library tests, eight opt-in checks excluded per ordinary suite),
doctests, strict demo Rustdoc, demo build, gettext and Node bridge/token checks
pass. Catalog edits only refresh references/extraction dates. The sixteen native
comparison paths and PNG dimensions pass; wider pairs differ only in the header
caption's 42×12-pixel region. Native narrow loaded/menu states were inspected.

The isolated Chrome 155.0.8059.85 check exercises all sixteen actual page states,
decodes each image, checks dimensions and Before/After/loaded state, and records
zero script errors or failed resource responses. The preceding toolbar's eight
states and index Next/Previous/direct links pass separately. This closes rendered
HTML verification for those pages, not live native motion or account acceptance.
