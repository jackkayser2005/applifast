# Apple toolbar beside a narrow Queue or Lyrics panel

At the 760-point window minimum, opening Queue or Lyrics could force the center
toolbar's right-to-left controls across Search. The existing minimum-width policy
only avoided this by asking the native window to grow, which fixed-size captures
and some window managers do not honor.

Apple mode now places its existing account, Settings, Now playing and mini-player
controls on a second row when the center pane cannot hold the full toolbar.
Navigation and Search keep the first row; a wider pane uses the original single
row. All actions remain available. Queue and Lyrics still start at the window top
and keep their full height. The panels retain their chosen widths and yield
temporarily as before. The minimum-width calculation keeps the existing page
space beside both panels instead of shrinking it to just the toolbar's controls.

The [native comparison](review-narrow-toolbar/index.html) has matching light/dark
and 760×760/1100×760 captures. The collection header/table crowding has a separate
[follow-up](narrow-collections.md). The maintainer approved both responsive scopes
on October 22, 2026: "Approve both layouts" (#82 and #83). Implementation and
release acceptance remain separate from that visual approval.

A new regression draws the real interface, finds the existing controls through
their accessibility nodes, and checks their painted bounds against the Search
pill and the center pane. The baseline fails with Settings overlapping Search
at 760 points with Queue open. The check covers both themes, Queue and Lyrics,
no panel, sidebar shown/hidden, and narrow/wide/narrow transitions. It also checks
that the page keeps at least its previous 270-point minimum. Native still images
do not establish live motion, real playback or the gaming resource budget.

No backend, queue ordering, credentials, storage format, dependency or network
destination changes. Legacy Spotify mode retains its single-row minimum policy.
Only Windows native captures are claimed; other platforms still require their
hosted compilation/test checks.

Local Windows checks pass: formatting, strict default/demo all-target Clippy,
all-target tests (1,040/1,087 library tests, eight opt-in checks excluded per
ordinary suite), doctests, strict demo Rustdoc, demo build, gettext and Node
bridge/token checks. Catalog changes only update references and extraction dates.
All eight comparison paths, PNG dimensions and Before/After state changes pass
a DOM-stub script check. Both 1100×760 pairs are pixel-identical; narrow changes
stay in the center pane. Representative native captures were inspected in both
themes. The browser connection failed before opening the review page, so rendered
HTML verification remains open. This is not a browser-test pass.
