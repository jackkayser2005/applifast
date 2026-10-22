# Interface scope approvals

Code checks and interface-scope approval are separate gates. This ledger records
new scope decisions; it does not replace the earlier approval notes in each
comparison or imply approval of unrelated work.

| PR | Visible scope | Evidence | Scope decision | Remaining implementation/release gates |
| --- | --- | --- | --- | --- |
| [#82](https://github.com/jackkayser2005/applifast/pull/82) | Two toolbar rows in narrow Apple center panes; wider panes retain one row. Queue/Lyrics stay full-height. | [Eight matching native captures](review-narrow-toolbar/index.html), both themes/sizes; normal pairs pixel-identical; actual UI geometry regression; Chrome matrix and index checks pass. | Approved October 22, 2026. Maintainer response to the combined #82/#83 scope question: "Approve both layouts". No adjustment requested. Retain across rebases preserving this scope. | Hosted checks on final head and real interactive use remain separate. |
| [#83](https://github.com/jackkayser2005/applifast/pull/83) | Smaller Apple collection header and separate filter row below 360 points; row names, favorites and More take priority over artwork/duration values. Songs/Favorites use Library captions at all widths. | [Sixteen matching native captures](review-narrow-collections/index.html), both themes/sizes and loaded/menu states; actual painted-glyph/control and pointer regression; Chrome matrix and index checks pass. Baseline includes the shared demo-only menu fixture correction. | Approved October 22, 2026. Maintainer response to the combined #82/#83 scope question: "Approve both layouts". No adjustment requested. Retain across rebases preserving this scope. | Hosted checks on final head, real interactive use and separate account/motion/resource acceptance remain open. |
