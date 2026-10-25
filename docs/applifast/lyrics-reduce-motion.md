# Stop a running lyric glide with Reduce motion

Turning Reduce motion on while lyrics are following a timed line now snaps to
the sung line in the side panel and full-screen view. Turning it off restores
gliding for later lines. The layouts, lyric source, account and playback commands
are unchanged.

Previously, the setting selected an instant duration for future scroll requests,
but a scroll target already running inside egui retained its earlier duration.
The fix reissues the current followed line once when the setting turns on and
uses the scroll area's existing instant-scroll option while it remains enabled.
It adds no dependency and leaves the existing manual-follow choice in place.

The regression renders the real application with a paused offline Apple fixture
and synthetic timed lyrics. At 1,100 and 760 pixels wide, in each lyric view, it
starts a glide, enables Reduce motion, checks the first paint after the scroll
request takes effect and checks thirty subsequent frames for rebound. It then
turns the setting off and verifies that a later line glides again. The fixture
hides the sidebar to retain room for the side lyric panel at narrow widths.

Restoring the original lyrics module from `d6f0add` with this exact test
reproduces continued movement after enabling the setting. The candidate module
is restored after that check. Native before-and-after capture and final-head
hosted checks remain pending for this follow-up.

This does not establish real-account Settings interaction, lyric availability,
audible transport, memory limits or the wider [release gates](release-readiness.md).
