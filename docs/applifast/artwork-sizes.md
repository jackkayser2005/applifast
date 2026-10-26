# Smaller Apple artwork for small surfaces

Apple resources previously supplied one 640-by-640 image to every display
target. Existing row and sidebar code could request a small cover, but had no
smaller variant to choose. The adapter now supplies 160-pixel thumbnails and
320-pixel collection images from Apple's documented size placeholders, while
retaining the original 640-pixel image first for headers and Home cards.
See [Apple's artwork contract](https://developer.apple.com/documentation/applemusicapi/artwork).

Both read routes participate. Native library events carry an optional
`artworkSmall` image through the MusicKit bridge. Rust catalog/collection parsing
supplies the same thumbnail, using the existing image models and selection
function. The player bar and track rows use their existing small-artwork path;
no layout, control, album choice or playback identifier changes. Fixed URLs are
preserved without guessing alternate CDN paths. Their original covers also get
locally generated thumbnails. Missing images keep their
existing placeholders.

The optional image is also stored in the existing library and queue/session
metadata. Its absence remains valid in old caches, and those entries retain
their original artwork until metadata refresh. The original large URL stays
unchanged. No old artwork cache is deleted automatically. Different requested
sizes have distinct entries in the existing hashed disk cache.

Fixed HTTP(S) covers use a client-only `#applifast-thumb-160` fragment as their
thumbnail cache identity. The server receives the original URL without that
fragment. The loader reuses original cached bytes when present, otherwise
downloads the original cover. It resizes off the UI thread to fit within
160 by 160 pixels, preserves aspect ratio and alpha, and never upscales.
Two workers bound simultaneous original downloads and decodes for this path.
Decode limits remain 8 MiB of encoded input, 8192 pixels per side and 64 MiB of
decoder allocation. The small PNG is atomically cached separately; existing
original files remain untouched. SVG URLs and URLs with existing fragments
retain their original fallback. Decode failures use the existing artwork error
and retry path. Custom library artwork is never replaced by a catalog cover.

This uses the existing artwork hosts and async loader. It can request additional
size-specific image assets, with no extra Apple metadata request, new service,
credential field, telemetry or audio persistence. Library and catalog identity,
uploaded-song playback parameters and unavailable-song handling are preserved.

The loader's existing decoded-image-plus-texture accounting models a square
640-pixel cover as 3.125 MiB and a 160-pixel cover as 0.195 MiB. That is a
93.75% reduction for a thumbnail entry; it is not a measured reduction in the
complete player's private memory. Existing large headers, retained caches,
drivers and the playback host still contribute. The [full-player resource
gates](release-readiness.md) remain open.

Regressions reproduce the original oversized thumbnail selection and lost small
artwork through the library bridge. They also check unchanged large images,
fixed/missing URLs, additive cache compatibility, uploads without catalog IDs,
and the existing display target choices. Ordinary demo data uses fixed artwork
URLs and exercises the compatibility path; it does not establish the quality
or resource footprint of Apple's new CDN variants.

The opt-in native read-only check restores protected authorization in a fresh
owned browser profile, reads library/catalog metadata and downloads a public
catalog artwork fixture at both sizes. It verifies decoded dimensions without
saving or displaying the image, audio playback, authorization popups, sign-out
or account writes.

The extended check also downloads one original library cover through the real
local-thumbnail loader and verifies its decoded dimensions, independently of
the provider thumbnail. It writes only to the fresh test profile's image cache,
which is removed after the check.

## Initial metadata-route verification

The native read-only check passes in 3.064 seconds on the working candidate
based on `864c17e`, with matching embedded page and bridge hashes. It reads 100
library songs and ten catalog songs. The public catalog fixture returns actual
640-by-640 and 160-by-160 images, at 118,170 and 9,845 encoded bytes respectively.
This verifies the requested image sizes and smaller transfer for that fixture.

The returned library batch has **zero small artwork variants**. A later native
check and regression revealed that the Windows bridge's allowlist discarded
`artworkSmall`. This count does not establish which artwork formats Apple
returned. The completed change adds a narrowly filtered thumbnail object at
that boundary, plus the local path for fixed covers. The earlier result does
not demonstrate library-thumbnail savings; the corrected runtime check and
whole-player measurements must be reported separately.

On that initial metadata-only candidate, local Windows formatting, strict
default/demo all-target Clippy, all-target
tests (1,035/1,081 library tests, eight opt-in checks ignored per ordinary suite),
doctests, strict demo Rustdoc, demo build, gettext and Node bridge/token checks
pass. The two model regressions and the bridge regression fail before their
corresponding changes and pass afterward. Standalone-host strict Clippy and
25 tests pass, with its native credential check excluded from the ordinary run.
Hosted all-feature/platform/site checks and whole-player measurements remain
separate. No new memory-optimized installer is qualified by these model and
asset-dimension checks alone.

## Completed thumbnail-path verification

The corrected native read-only check passes in 6.220 seconds, with matching
embedded page and bridge hashes. All 100 library songs now carry a small image
through the Windows boundary. The selected library thumbnail is 160 by 160 at
6,147 encoded bytes. Explicitly resizing that same original library cover
through the local loader also produces 160 by 160, at 24,613 PNG bytes. The
catalog fixture again returns 640 and 160 pixels at 118,170 and 9,845 bytes.
The local fixture tests the real original cover independently of its available
CDN thumbnail; it does not establish that this account batch automatically
selected a fixed-URL cover. Exact source/binary hashes and scope are recorded in
[the native result](artwork-thumbnail-check.json). No private artwork or URLs
are published.

The fixed-cover HTTP regression fails before the loader change because it
decodes 640 by 320 pixels instead of 160 by 80. Afterward it also verifies
transparency, a 160-by-80 texture through the ordinary egui loader, and cache
reload after the source server exits. Additional tests verify original-cache
reuse without network or overwrites, no upscaling, source/query preservation,
and waiting when both resize slots are occupied. The native bridge regression
fails before its allowlist change and passes afterward, preserving unavailable
uploaded-song metadata while filtering extra fields.

Final Windows checks pass: formatting, strict default/demo all-target Clippy,
all-target tests (1,038/1,084 library tests, eight opt-in checks excluded in each
ordinary suite), doctests, strict demo Rustdoc, demo build, gettext and Node
bridge/token checks. The standalone host passes strict all-target Clippy and
26 tests, with its separate native credential check excluded. The read-only
account artwork check above is run explicitly. Other-platform and all-feature
checks remain hosted CI gates, not claims of local runtime coverage.

These are artwork correctness checks, not whole-player resource acceptance.
The installed `3508c61` preview predates this artwork change. Latest-candidate
minimized playback/paused samples and matched native cover-quality review
remain pending.
