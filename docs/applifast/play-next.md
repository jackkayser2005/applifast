# Play next in Apple mode

The `feat/apple-play-next` follow-up adds **Play next** to song, selected-song
and album menus. It prepends to the manual queue without restarting the current
track. Selected rows keep their displayed order and repeated occurrences.
**Add to queue** continues to append behind manual additions. Upcoming queue
rows retain their existing jump, add-copy and removal controls.

Albums use the existing asynchronous metadata reads. All pages must succeed
before an album is inserted. Album requests complete in click order, so the
latest **Play next** album is first even when its metadata arrives earlier.
A failed older album releases later ready albums. Clear queue and sign-out
cancel pending choices. Double-click protection distinguishes **Play next**
from **Add to queue** on the same album.

This uses existing typed Apple identifiers, queue commands, intent generations
and local session storage. It adds no dependency, credential, storage format,
network destination, hosted service or Apple playlist mutation. Spotify mode
does not expose the new command.

## Verification

Focused tests cover front insertion versus append, current uploaded-library
identity and seek position, repeated selections, immediate Next, complete and
failed album pagination, out-of-order album reads, cancellation and menu clicks.
Matching light/dark and narrow/normal native captures are provided in the
[HTML comparison](review-play-next/index.html). Captures use identical
demo-only menu fixtures on the `0745e94` baseline and candidate.

The native-account test was extended to prepend a manual duplicate while
MusicKit keeps playing, verify the actual position is preserved, then verify
Next selects that occurrence before the remaining context row and pause.
This extension has not passed on the current account: both the demo-feature
and final default-feature harness restored authorization and loaded the SDK,
then MusicKit rejected the first library read before any playback command.
They failed after 3.71 and 4.74 seconds respectively, with the sanitized message
“MusicKit rejected an operation. Check authorization, subscription, connection,
or the selected item.” No popup, sign-out, favorite or playlist write was sent.
The previous queue-removal native pass does not establish this extension.

The native credential-store round trip passes separately using only unique
dummy grants. Windows formatting, strict default/demo all-target Clippy,
default/demo all-target tests, doctests and strict demo Rustdoc pass. The library
suites pass 1,023 and 1,065 tests, with six opt-in checks ignored in each ordinary
run. Four Flatpak metadata tests pass. Linux launcher fixtures require unavailable
Ruby/POSIX tools, and one release-name fixture needs Windows symlink privilege;
these environment failures remain recorded. All-feature, Jekyll and Nix checks
need the hosted environments.

The demo build, gettext consistency check and bridge/token-signer self-checks
pass. All fourteen complete catalogs translate the new label; an independent
comparison verifies that existing translations remain unchanged. All 24 native
captures were inspected, with matching dimensions and the added menu row visible
in each pair. The HTML's selectors and Before/After handlers pass a static Node
VM check. Browser rendering is unverified because the browser-client bootstrap
previously failed with EPERM here; no alternative automation was used. Both
owned native test browser profiles were removed after shutdown.

Fresh-PC sign-in, audible desktop use, interactive motion, rendered HTML and
gaming acceptance remain separate release gates.
