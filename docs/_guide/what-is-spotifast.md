---
redirect_from: /what-is-fastpotify/
title: What is Applifast?
description: Apple Music on Windows with a native interface, background playback and a compact mini player.
nav_order: 0
---

## Why Applifast

**Apple Music, with a native Windows interface.** Applifast keeps browsing and
desktop controls in Rust and egui. An independent MusicKit JS/WebView2 host
handles Apple authorization and playback.

It is built for listening while other apps and games are running. It is
currently a development preview for invited Windows x64 testers.
The [release checklist](/applifast/release-readiness.html) records measurements
and remaining acceptance checks. There is no guaranteed whole-player memory
figure yet.

You need an Apple Music subscription and internet access. The preview supplies
the app's developer token, so you can authorize your own Apple account without
joining the Apple Developer Program.

![Applifast Apple library with deterministic sample data](/applifast/review-narrow-collections/after-loaded-dark-1100x760.png)

## What it does

- **Library and search.** Browse synced songs, albums, artists and playlists,
  plus catalog and library search. Home shows supported Apple shelves.
- **Daily listening.** Play, pause, seek, change volume, shuffle and repeat.
  Arrange a local queue with Play next, Add to queue and separate repeated songs.
- **Desktop controls.** Listen from the tray, use media keys and switch between
  the main window and mini player.
- **Favorites.** Add songs to Apple Favorites, filter favorites and choose how
  many loaded favorites appear in the left sidebar.
- **Playlists.** Create playlists and append supported songs. Pin library
  playlists locally near the top of the sidebar.
- **Appearance and lyrics.** Choose a theme, enable Reduce motion and open
  full-screen Now Playing. Lyrics come from LRCLIB when available.

These controls are implemented. The [preview checklist](/applifast/testing.html)
distinguishes automated evidence from real-account and desktop acceptance still
needed before a public beta.

## Your library stays yours

Library IDs and catalog IDs stay distinct. Applifast preserves Apple's playback
parameters and never silently substitutes a catalog recording for an upload.
A song without usable playback parameters stays visible as unavailable.

The app caches library metadata and artwork and restores the local queue
paused after restart. This does not provide offline audio downloads.

## Current limits

- Apple playback is Windows-only. Linux and macOS do not have Apple engines.
- Cloud-only uploads depend on Apple supplying usable playback parameters.
  Universal upload playback has not been established.
- Playlist creation and appending do not imply rename, delete, playlist-track
  reordering, folders, cover editing or collaboration.
- Apple mode has no Spotify Connect, podcasts, PCM spectrum, oscilloscope,
  equalizer or MilkDrop. Ambient Pulse and playing indicators are decorative.
- Lyrics come from LRCLIB, not Apple; timing and coverage vary.
- Private previews are unsigned, have an expiring app token and update through
  replacement installers. Public distribution is still pending.

## Open source and credit

Applifast is an MIT-licensed fork of
[Spotifast by Carmine Paolino](https://github.com/crmne/spotifast).
It retains upstream credit and builds on Rust, egui and fastframe.
The Windows port uses Apple's MusicKit JS and Microsoft's WebView2.

There is no Applifast account, hosted backend, app telemetry or decrypted-media
cache. Read [Privacy](/privacy/) for storage and network behavior.

Applifast is independent of Apple and is not affiliated with or endorsed by Apple.
Report problems on [the fork's issue tracker](https://github.com/jackkayser2005/applifast/issues).
