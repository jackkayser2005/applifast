# Apple library continuation page size

Apple can return a next-page cursor containing only the offset, without the
initial page limit. Its [pagination guide](https://developer.apple.com/documentation/applemusicapi/fetching-resources-by-page)
documents supplying the limit again. The bridge forwarded those cursors without
reapplying the app's existing 100-item library page size, increasing the requests
needed for a complete collection.

The shared MusicKit read boundary now supplies `limit=100` for top-level library
songs, albums, artists and playlists when no explicit limit is present. It keeps
explicit limits and excludes ID batches and mutations. Search, details and
relationship reads retain their existing sizing. It adds no endpoint, network
destination, dependency, credential handling, storage format or visible layout.
Read deadlines, cancellation and stale-answer handling remain in place.

## Actual saved-account result

On October 22, 2026, a fresh owned Windows browser profile restored the protected
grant and read complete collections. The fixture then used the production
`Library` command and Rust `State` coordinator to accumulate songs separately,
comparing every library ID and playback-parameter object with the raw reads.
The complete check passed in 181.90 seconds (182.138 seconds including its driver).
It sent no playback, account-write, popup or sign-out command, and left the local
playback queue empty. Only counts and source/binary hashes were retained in the
[sanitized evidence](library-page-size-evidence.json), without resource IDs,
song names or authorization values.

| Collection | Resources | Before pages | After pages |
| --- | ---: | ---: | ---: |
| Songs, raw reads | 3,698 | 145 | 37 |
| Albums | 2,851 | 112 | 29 |
| Artists | 1,786 | Unfinished | 18 |
| Playlists | 94 | Unfinished | 1 |
| Songs, production Library/state | 3,698 | Unfinished | 37 |

The initial five-minute cycle stopped after completing only songs and albums.
Its partial results and timeout remain recorded. It is not a matched full-cycle
latency baseline, so no end-to-end speedup percentage is claimed. The two
completed collections have roughly 74% fewer page requests after the fix.
The passing run confirms that incoming second-page cursors omit the limit while
the returned pages contain 100 resources.

All four collection models retain their library identities. Both songs paths
preserve the same original IDs and admitted playback parameters. Of the songs,
130 have no catalog ID and 125 have no playback parameters. Those 125 entries
remain in the completed state rather than being discarded or replaced by a
catalog song. These metadata categories do not establish matched-file or
genuine uploaded-file provenance, and available parameters do not prove playback.

The source base is `7d8c82f`; the native binary was built with the changed bridge
and Windows-only opt-in fixture. Their exact hashes are in the evidence. The
fixture accepts only same-collection continuation routes, detects repeated
paths/IDs and empty continuation pages, and stops at 200 pages per collection.
It refuses a normal browser profile and uses a new child of the checkout's
`.cache/library-pagination-qa`. The driver has a five-minute overall deadline.
Those are QA bounds, not Apple API limits or application library limits.

## Regression scope and remaining gates

The Node bridge check fails before the fix because the SDK call for a songs
continuation has no limit. Afterward it covers the typed Library path and all
four generic collection reads, preserved explicit limits, unchanged detail and
search sizing, and the existing exact playlist/favorite mutation and ID-batch
checks. The Rust continuation guard rejects malformed, external and
cross-collection cursors without sending a further request. Normal test suites
ignore the live account fixture.

Windows checks pass: formatting, strict default/demo all-target Clippy, all-target
tests (1,042 default and 1,091 demo library tests, with ten opt-in tests ignored
per ordinary suite), doctests, strict demo Rustdoc, demo build, gettext and the
Node bridge/token checks. The standalone Windows host also passes formatting,
strict all-target Clippy, its ordinary tests (28 passed, three opt-in ignored)
and build. The live read above ran separately from the ordinary suites. Hosted
checks for this follow-up and other-platform compilation are still pending.

This closes the stated saved-account collection traversal and model/coordinator
identity checks for this library. It does not establish fresh authorization,
catalog/library search pagination, desktop load-more controls, Home dates,
refresh/restart behavior, click-to-audio latency, audible/cloud-only playback,
account lifecycle or the memory/CPU budget. Those remain in the
[release checklist](release-readiness.md). The existing `7d8c82f` private installer
predates this network follow-up.
