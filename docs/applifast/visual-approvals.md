# Interface scope approvals

Code checks and interface-scope approval are separate gates. This ledger records
new scope decisions; it does not replace the earlier approval notes in each
comparison or imply approval of unrelated work.

| PR | Visible scope | Evidence | Scope decision | Remaining implementation/release gates |
| --- | --- | --- | --- | --- |
| [#82](https://github.com/jackkayser2005/applifast/pull/82) | Two toolbar rows in narrow Apple center panes; wider panes retain one row. Queue/Lyrics stay full-height. | [Eight matching native captures](review-narrow-toolbar/index.html), both themes/sizes; normal pairs pixel-identical; actual UI geometry regression. | Pending maintainer approval. No adjustment requested yet. | Hosted checks on final head, rendered HTML, real interactive use. Narrow collection/table crowding remains open. |
