---
title: The Mini Player
description: Keep Apple Music playback and its upcoming queue in a compact window with classic skins.
nav_order: 4
---

Open the mini player with **Ctrl+M**, the shrink control or
**Switch to it** in Settings. Use the shortcut or the return control to reopen
the main window. Both windows use the same playback engine and local queue.

![The Apple-mode mini player with sample data](/applifast/review-mini-heart-motion/after-ready-dark-550x232.png)

## Playback and upcoming songs

The mini player shares play/pause, seek, volume, previous/next, shuffle and
repeat with the main window. Its heart and **B** shortcut add eligible songs
to Apple Favorites, using the same pending/error handling.

**PL** opens the upcoming playlist below the player. It includes the playing
song and coming occurrences. Double-click an upcoming song to choose it;
repeated songs remain separate. Use **REM > Remove selected** for selected
upcoming occurrences; the playing row is protected. Remove all clears manual
additions, not the entire remaining context.

The Add menu opens Apple Music search or Favorites. Drag the lower-right
corner to resize the playlist. Closing this panel does not clear the queue.

## Skins and size

Classic Winamp 2 `.wsz` skins can change the appearance. Drop a skin on a
window to install it, or choose an installed skin in Settings.
The [Winamp Skin Museum](https://skins.webamp.org) is an external site.

The title-bar menu offers scales from 1x to 4x and always-on-top.
**D** toggles double size and **A** toggles always-on-top.
Shade controls roll panels into thin bars. Settings retain the mini player's
own size and position separately from the main window.

Winamp 3 and 5 skin formats are not supported. A skin changes appearance;
it does not add playback capabilities.

## Decorative display

Apple-mode Ambient Pulse uses playback state, progress and album-art colors.
It is independent of the volume control. Reduce motion stills it.
This is not an audio spectrum or oscilloscope.

MusicKit does not expose PCM to this engine. Equalizer, mono, balance,
oscilloscope, spectrum and MilkDrop controls are therefore inactive or absent.
Artwork printed on a classic skin does not make those functions available.

## Tray and restart

Closing to the tray can keep music playing. Reopen from the tray icon and
use **Quit** to exit completely. The saved account-bound queue restores paused
after restart. See [Everyday Use](/using-spotifast/) for queue ordering and pins.

Window recreation during audible playback remains in the
[preview checklist](/applifast/testing.html), separately from deterministic
layout and interaction checks.
