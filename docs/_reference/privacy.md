---
title: Privacy
description: What the Windows Apple Music preview stores, which services it contacts and what it never collects.
nav_order: 4
---

Applifast has no account of its own, hosted backend, app telemetry, analytics
or advertising. Music playback still contacts Apple's services under your
authorized account.

## What stays on your computer

- **Authorization.** App and user tokens use Windows Credential Manager under
  the separate Applifast playback service. A private WebView2 profile also keeps
  browser session data. Do not share that profile, protected-store exports or
  authorization responses.
- **Settings.** Theme, window preferences, local playlist pins and the
  Favorites shelf selection remain on this PC.
- **Library and queue.** A non-secret account-bound snapshot retains song
  metadata, original playback identifiers, queue occurrences and position.
  Restart restores paused. Metadata can reveal listening preferences even though
  it contains no authorization token.
- **Caches.** Artwork and LRCLIB lyrics use local caches. The app does not save
  decrypted audio or provide offline Apple Music downloads. WebView2 and
  MusicKit maintain their own browser/runtime-managed data.
- **Logs.** Logs remain local; they are not uploaded automatically. Credentials
  and authorization responses must not be logged. Review a diagnostic before
  sharing it, since normal metadata or local paths may still be personal.

[Settings & Files](/settings-and-files/) lists the Windows paths.
Use **Sign out** in Settings to clear saved user authorization and request
browser-session/account-cache clearing. A reported clearing failure needs
attention before reauthorization. Uninstalling program files alone does not
sign out or erase user data.

## What is sent, and to whom

- **Apple.** MusicKit's SDK, authorization, library/catalog requests, Favorites,
  supported playlist writes, artwork and playback use Apple's services and
  content-delivery hosts. The library's original artwork URL is preserved.
  Apple's own service terms and privacy policy apply.
- **Microsoft.** The app uses the installed Evergreen WebView2 Runtime.
  Runtime installation, servicing and its browser behavior are Microsoft's
  components; no App telemetry claim here overrides their policies.
- **LRCLIB.** Opening lyrics can send the song's artist, title, album and
  duration to [lrclib.net](https://lrclib.net). No Apple authorization token or
  account identifier is sent for that lookup.
- **Links you choose.** External links open in your normal browser. Their
  websites receive the usual browser traffic.

The playback page is mapped locally to `https://applifast.invalid`.
That name is a local WebView2 origin, not an Applifast-hosted server.
Apple mode does not perform the retained Spotify Connect discovery or upstream
Spotifast update checks. It does not fetch MilkDrop preset packs.

## This documentation

The inherited website build and deployment are disabled for this fork.
No new Applifast public website or analytics service is activated by these
documentation changes. Upstream Spotifast's site has its own policies.

## Questions

Use [the fork's issue tracker](https://github.com/jackkayser2005/applifast/issues).
Never attach signing keys, developer tokens, listener tokens or browser profiles.
