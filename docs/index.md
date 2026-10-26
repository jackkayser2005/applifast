---
layout: home
title: Applifast
description: Apple Music for Windows with a native interface, a local queue and a compact mini player.
permalink: /
hero:
  name: Applifast
  text: Apple Music on Windows
  tagline: Browse your library, arrange your queue and listen from a native Rust interface, the tray or a compact mini player.
  actions:
    - theme: brand
      text: Windows preview
      link: /download/
    - theme: alt
      text: What is Applifast?
      link: /what-is-spotifast/
    - theme: alt
      text: GitHub
      link: https://github.com/jackkayser2005/applifast
  image:
    src: /applifast/review-narrow-collections/after-loaded-dark-1100x760.png
    alt: Applifast Apple library with deterministic sample data
    width: 1100
    height: 760
features:
  - icon: ⚡
    title: Built for desktop listening
    details: Rust and egui handle the interface. An independent MusicKit and WebView2 host handles Apple playback. Resource measurements remain part of preview testing.
  - icon: 📚
    title: Your Apple library
    details: Browse synced songs, albums, artists and playlists, with catalog and library search. Original library identities stay distinct from catalog matches.
  - icon: ⏭️
    title: Choose what plays next
    details: Play next, add to queue, reorder upcoming occurrences and restore the saved session paused after restart.
  - icon: ❤️
    title: Favorites and playlists
    details: Add Favorites, choose a compact sidebar shelf, create playlists, append songs and keep local playlist pins.
  - icon: 🎨
    title: Themes and motion
    details: Choose light, dark or custom colors and Reduce motion. Ambient Pulse is decorative, not an audio spectrum.
  - icon: 📻
    title: Compact mini player
    details: Keep playback and an upcoming queue close at hand. Classic Winamp skins change the appearance while both windows share one player.
    link: /winamp/
    link_text: Mini-player guide
  - icon: ⌨️
    title: Desktop controls
    details: Use shortcuts and media keys, close to the tray and reopen while the same playback host keeps running.
  - icon: 🔓
    title: Open source
    details: An MIT-licensed fork of Spotifast by Carmine Paolino, retaining upstream credit. No Applifast hosted backend or app telemetry.
    link: https://github.com/jackkayser2005/applifast
    link_text: Read the source
---

## Try the Windows preview

This is an invited **Windows x64 development preview**, not a published beta.
You need an Apple Music subscription, internet access and Evergreen WebView2.
The package supplies the app developer token; sign in with your own Apple
account. No developer membership or command-line setup is needed for testers.

[Getting Started](/getting-started/) explains installation and Apple authorization.
[Download](/download/) describes the current private distribution and checksums.

## Keep listening your way

The main window, mini player and tray share playback and a local queue.
Choose a coming song, keep duplicate occurrences, pin library playlists and
show a few loaded Favorites in the sidebar. Full-screen Now Playing can show
LRCLIB lyrics when available. [Everyday Use](/using-spotifast/) covers the controls.

Apple mode does not expose Spotify Connect or PCM spectrum, oscilloscope,
equalizer and MilkDrop features. Unavailable uploads remain visible instead
of being replaced with catalog recordings. Linux and macOS Apple playback
are unsupported.

## Follow the work

Private preview checks and remaining public-beta gates are recorded in the
[release checklist](/applifast/release-readiness.html).
See [Privacy](/privacy/) for storage and service traffic.

Applifast builds on [Spotifast](https://github.com/crmne/spotifast),
[egui](https://github.com/emilk/egui) and
[fastframe](https://github.com/crmne/fastframe).
It is independent of Apple.

<style>
  .VPHero .image-container { width: 100% !important; height: auto !important; transform: none !important; }
  .VPHero .image-src { position: relative !important; top: auto !important; left: auto !important; transform: none !important; width: 100% !important; height: auto !important; max-width: 100% !important; max-height: none !important; padding: 0 !important; border-radius: 12px; box-shadow: 0 12px 48px rgba(0, 0, 0, 0.45); }
  @media (max-width: 959px) { .VPHero .image { margin: 0 0 24px !important; } }
</style>
