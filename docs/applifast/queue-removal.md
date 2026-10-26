# Remove individual Apple queue occurrences

The main queue context menu offers **Remove from queue** for an upcoming row.
The mini-player playlist offers **REM > Remove selected** for selected upcoming
rows, excluding the playing row. Its ADD menu uses **Search Apple Music** and
**Favorites** in Apple mode. Spotify controls keep their previous behavior.

Removal targets queue positions, not song IDs. A manual duplicate and a context
duplicate can be removed independently. Removing a context occurrence also
removes it from this session's shuffle/repeat context, so it does not return
on the next cycle. Current/history occurrences remain intact. A deliberate
removal permits immediately adding that song again. **Remove all** continues
to clear manual additions only.

The existing queue-snapshot command updates the independent host without
restarting the current song. Host, account and playback-intent generations
reject delayed actions. Mini-player selections clear when the displayed queue
changes; volume changes preserve them. Existing version-1 account-scoped
metadata/session caching stores the resulting order and restores it paused.
No credential storage, network destination, dependency or audio cache is added.
These controls never delete a song or edit a remote playlist.

## Verification

Focused regressions cover duplicate manual/context removal, current upload
identity and position, atomic rejection of invalid batches, repeat/shuffle,
immediate re-addition, all three stale-action generations, replayed actions,
restart restoration and actual menu clicks. The mini-player interaction test
selects the playing row with two manual copies and confirms only the upcoming
copies are removed; it also checks selection invalidation and Apple/Spotify
navigation wording.

The [native comparison](review-queue-removal/index.html) uses matching baseline
production code from `7d25258` and this candidate, both with identical demo-only
menu fixtures. Main-queue captures cover light/dark and normal/narrow sizes;
mini-player captures cover both themes at skin scales 1 and 2, including ADD
and REM menus. Native captures and deterministic interactions do not establish
fresh authorization, audible playback or rendered HTML review.

The isolated Windows native-account test passes in 45.43 seconds. It restores
protected authorization, reads 100 library songs and real library/catalog
shelves, plays muted, seeks, pauses and resumes at the saved position. It then
publishes three occurrences of the same library song, resumes, removes one
upcoming occurrence through the production queue state, and observes the host
still playing at or beyond its prior actual position with the other occurrence
queued. It finishes paused. The owned workspace browser profile was removed
after shutdown. No popup, sign-out, Favorites or playlist-write command was sent.
This does not establish audible desktop listening, genuine cloud-only uploads,
fresh sign-in, window recreation or gaming performance.

Windows formatting, strict default/demo all-target Clippy, default/demo
all-target tests, doctests, strict demo Rustdoc, demo build, gettext and Node
bridge/signer checks pass. The library suites pass 1,021/1,062 tests, with six
opt-in tests ignored in each; the native-account test passes separately above.
All fourteen complete catalogs translate the new command, and an independent
comparison verifies that existing translated messages remain unchanged.
Flatpak metadata tests pass. Linux launcher fixtures require missing Ruby/POSIX
tools, and one release-name fixture needs Windows symlink privilege; those
environment failures remain recorded, without skipped tests or weakened checks.
Local all-feature, Jekyll and Nix checks require the hosted environments.

All 24 captures were inspected and have matched dimensions with visible changes.
The HTML's 24 selector paths and Before/After handlers pass a static Node check;
browser rendering is unverified because the browser-client bootstrap previously
failed with EPERM in this environment. No alternative automation was used.
Optimized package verification is required before the new preview is handed off;
results and artifact hashes accompany the feature PR. Hosted CI, visual approval
and the fresh-PC family trial remain separate gates.
