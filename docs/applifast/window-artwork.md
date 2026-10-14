# Artwork ownership when windows are recreated

The main window and mini player are recreated around long-lived application
state. The mini-player's skin textures were already cleared on attachment,
but softened covers and the lyrics backdrop still held textures from the old
egui context. Texture IDs are local to their manager; retaining such a handle
does not upload its image to the new renderer. A reused numeric ID can refer
to a different image there.

`App::attach` now resets those two window-owned caches with the existing skin
cleanup. This drops old handles, pending receivers and failure markers. Results
still arriving for the closed window cannot enter the fresh caches. Later draws
create textures in the new context, using the existing artwork loader and disk
cache. The playback host, account, library, queue, lyrics content and persistent
settings are not reset. No dependency, credential format, cache schema or new
network destination is introduced.

## Regression and acceptance

The regression first fails because the old texture manager is still alive
after window recreation. It passes with the fix across repeated main/mini-player
attachments, checking real egui texture handles and a weak reference to the old
manager. It also verifies that the old backdrop/cover completion channels are
disconnected, failed/pending entries are cleared, and a newly completed cover
is registered with the new context's manager.

Windows formatting, strict default/demo all-target Clippy, default/demo
all-target tests, doctests, strict demo Rustdoc, demo build, gettext consistency
and Node bridge/token-signer checks pass. Library suites pass 1,025 default and
1,069 demo tests, with six opt-in checks ignored in each ordinary run. Translation
updates change generated source references only. No credential-storage code
changed, and no native account or authorization command was sent for this fix.
Other-platform and all-feature results require the candidate's hosted CI.

The [HTML comparison](review-window-artwork/index.html) covers matching native
Windows light/dark, normal/narrow captures of the lyrics panel, Now Playing and
mini-player playlist. These are fresh-window appearance checks, not evidence
of interactive native window recreation or animation. The lifetime regression
checks the recreation path separately. HTML browser rendering remains a
separate acceptance check.

All 24 captures were inspected. After matching the account-avatar hover state,
all twelve before/after pairs are pixel-identical. A static Node VM verifies
the 24 selector paths and Before/After controls; it does not render the HTML.

The [integrated memory baseline](integrated-performance.md) was measured on the
older `0745e94` preview. This targeted lifetime correction does not establish
how much of that native footprint came from artwork, demonstrate a memory
reduction, or pass the resource budget. Latest-build controlled measurements,
actual tray/main/mini-player recreation with audible playback, and the gaming
session remain required.
