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
preserved without guessing alternate CDN paths. Missing images keep their
existing placeholders.

The optional image is also stored in the existing library and queue/session
metadata. Its absence remains valid in old caches, and those entries retain
their original artwork until metadata refresh. The original large URL stays
unchanged. No old artwork cache is deleted automatically. Different requested
sizes have distinct entries in the existing hashed disk cache.

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

## Metadata-route verification and remaining library work

The native read-only check passes in 3.064 seconds on the working candidate
based on `864c17e`, with matching embedded page and bridge hashes. It reads 100
library songs and ten catalog songs. The public catalog fixture returns actual
640-by-640 and 160-by-160 images, at 118,170 and 9,845 encoded bytes respectively.
This verifies the requested image sizes and smaller transfer for that fixture.

The returned library batch has **zero small artwork variants**. This provider
metadata route therefore does not demonstrate a library-thumbnail memory saving
on this account. Fixed and missing URLs retain their existing fallback. A
bounded local-thumbnail path for fixed library artwork remains required before
calling this feature complete; catalog artwork must not replace custom covers.

Local Windows formatting, strict default/demo all-target Clippy, all-target
tests (1,035/1,081 library tests, eight opt-in checks ignored per ordinary suite),
doctests, strict demo Rustdoc, demo build, gettext and Node bridge/token checks
pass. The two model regressions and the bridge regression fail before their
corresponding changes and pass afterward. Standalone-host strict Clippy and
25 tests pass, with its native credential check excluded from the ordinary run.
Hosted all-feature/platform/site checks and whole-player measurements remain
separate. No new memory-optimized installer is qualified by these model and
asset-dimension checks alone.
