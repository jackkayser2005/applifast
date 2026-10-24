---
redirect_from: /using-fastpotify/
title: Everyday Use
description: Browse Apple Music, arrange your queue, use Favorites and playlists, and listen from the mini player.
nav_order: 3
---

## Find music

Home shows the Apple shelves available to your account. Open Songs for synced
library songs, or browse albums, artists and playlists from the sidebar.
Search includes catalog and library resources; their identities stay separate
even when titles match.

Use type filters and **Load more** where available. Filtering a loaded
collection does not search the full catalog. **F5** refreshes music pages.
A failed page keeps loaded rows and offers retry.

## Start and resume

<a id="recent"></a>

Home's Recently played and Recently added shelves use the Apple data available
to your account. Refresh with **F5**. They do not promise the complete history
provided by an upstream Spotify feature.

Double-click a playable song to start at that row. The bottom player bar or
**Space** pauses and resumes the current song. Shuffle and repeat change local
playback order. The main window, mini player and desktop controls share state.

A failed song retains its queue entry and shows an error. An unavailable upload
is not replaced with a catalog match.

## Choose what plays next

Right-click a song, selection or album:

- **Play next** goes before existing manual additions.
- **Add to queue** appends after manual additions.
- Context tracks follow manual additions.

Album additions wait for all required pages. Repeated songs stay separate
occurrences. Select an upcoming occurrence, drag/reorder it, remove that
occurrence or clear manual additions from the queue. These local actions do
not reorder an Apple Music playlist.

Quit and reopen restores the session paused, including current occurrence and
position. The snapshot belongs to the signed-in account.
See the [queue contract](/queue/) for detailed ordering.

## Favorites

Use the heart, a song menu or **B** to add an eligible song to Apple Favorites.
The heart updates immediately while confirmation is pending; a failed write
shows an error and rolls back. Existing or pending favorites do not submit
another write.

Open **Favorites** or use the favorites-only filter. The left-sidebar shelf
shows a configurable number of loaded favorites, up to ten; zero hides it.
**Show all favorites** opens the page. Unknown flags are not assumed false.
Favorites are separate from star ratings.

Removing a favorite is not exposed without a verified supported route.

## Playlists and pins

Use **Add to playlist** for supported songs, selections, albums or queue items.
Choose **New playlist** to create one. The queue can also be saved as a new
playlist. Confirm a known duplicate if you want another copy.

Apple can take time to show a completed write. Applifast retains pending local
state while checking. If the result is uncertain, inspect the playlist before
submitting again; automatic write retries could create duplicates.

Open a loaded library playlist and choose **More > Pin to top**, or right-click
it in the sidebar. **Unpin** removes the pin. Drag within the pinned block to
arrange it. Pins stay on this PC and do not alter Apple's playlists.

Rename, delete, playlist-track reorder, folders, covers and collaboration are
not supported by this slice.

## Mini player and tray

**Ctrl+M** switches windows. Both use the same queue and playback engine.
Choose an upcoming song from the mini-player queue. Classic `.wsz` skins can
change its appearance. Unavailable audio controls remain inactive or absent
in Apple mode.

Closing to the tray can leave music playing. Reopen from the tray icon; use
**Quit** before installing a new preview or exiting completely.

## Lyrics and motion

Open Lyrics or full-screen Now Playing for available LRCLIB lyrics. Timed lines
follow the playhead; selecting a timed line seeks the song. Instrumental,
missing and failed lookups have their own states. These are not Apple's lyrics,
and coverage and timing vary.

**Reduce motion** stills transitions and decorative indicators. Ambient Pulse
uses playback state, progress and artwork colors, independently of volume.
It does not analyze audio.

## Shortcuts and scrolling

Open **Settings > Keyboard shortcuts** or press **Ctrl+/** for the current list.
**Alt+1** through **Alt+5** open Home, Songs, Favorites, Albums and Artists.
**Ctrl+Shift+Q** opens the queue and **F5** refreshes.
Text fields keep normal editing shortcuts.

Hold **Shift** while turning the wheel over a horizontal shelf to scroll it.
Windows middle-click autoscroll starts in a scrolling list's empty background
and stops with another click, Escape, the wheel or a window switch.

For installation, use [Getting Started](/getting-started/).
For connection problems, use the [tester guide](/applifast/testing.html).
