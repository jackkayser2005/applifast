---
title: Getting Started
description: Install the Windows preview, sign in with Apple, and start listening.
nav_order: 2
---

## Install

Applifast is currently an invited **Windows x64 development preview**.
Get the installer or portable ZIP from the maintainer; see
[Download](/download/) for availability. Quit any older build through its tray
menu first. Closing its window can leave it running.

Run the installer and open **Applifast** from Start. It installs the app and
companion developer token together, without an administrator account.
For the portable version, extract the entire ZIP and run **Applifast.exe**.

Current previews are unsigned. Compare the provided SHA-256 checksum before
opening a download. A matching checksum checks file integrity, not publisher
identity.

## Sign in

You need an Apple Music subscription, internet access and the
[Evergreen WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

Click **Sign in with Apple** and complete Apple's authorization popup.
There is no browser code to extract. Enter your password only on Apple's page.
Applifast saves its app and user authorization in Windows Credential Manager.
Its separate WebView2 profile also holds browser session data.

The preview supplies the app's developer token. Listeners do not need Developer
Program membership, a signing key, Rust or Node. Keep **developer-token.txt**
beside **Applifast.exe**; the installer does this for you. Each person authorizes
their own Apple Music account.

Use **Cancel** if you change your mind. Closing the popup cancels that attempt
too. Fresh-PC authorization, including close/cancel/retry, remains part of
preview acceptance.

## Start listening

Open Songs, Albums, Artists, a library playlist or Search. Double-click a
playable song or use a collection's Play button. The bottom player bar controls
play/pause, seeking, volume, shuffle and repeat.

Library songs keep their own Apple IDs and playback parameters. A song with no
usable parameters stays visible as unavailable. Applifast does not replace an
upload with a catalog recording.

- Closing to the tray can keep music playing. Reopen from the tray icon;
  use its **Quit** command to exit completely.
- **Ctrl+M** switches to the mini player. Both windows share playback and queue.
- **Space** plays and pauses. Text fields keep their normal typing behavior.
- Right-click for **Play next**, **Add to queue**, Favorites and playlist actions.
- **F5** refreshes music pages. Cached rows remain visible during refresh.
- Quit and reopen restores the saved queue paused.

See [Everyday Use](/using-spotifast/) for Favorites, pins, lyrics and shortcuts.

## Appearance

Open **Settings > Appearance** to choose your theme and language.
**Reduce motion** stills decorative transitions. The main and mini player
share playback state.

## If connection or sign-in fails

Check the network, quit other Applifast builds through their tray menus and
choose **Retry connection**. Install or repair Evergreen WebView2 if missing.
An installed Edge browser alone is not the production runtime.

An expired app token needs a replacement from the maintainer. Use
**Settings > Account > Import developer token** to choose its text file.
Never select a `.p8` signing key. Retry cannot renew a token.

For rejected account authorization, check your subscription and use Sign out
before authorizing again. This interrupts playback and clears account-specific
local state. Do not delete an active browser profile manually.

The [tester guide](/applifast/testing.html) includes detailed recovery steps.
Reports should name the source revision in **BUILD.txt**, Windows version and
steps to reproduce the problem. Never attach tokens, keys or browser profiles.

## Build from source

Contributors need the repository's pinned Rust toolchain and Visual Studio C++
build tools. In a checkout of
[the fork](https://github.com/jackkayser2005/applifast):

```powershell
cargo run --release --locked
```

A checkout without a companion developer token needs protected local token setup.
See [the tester/setup guide](/applifast/testing.html).
Default Windows builds do not enable retained MilkDrop code. Linux and macOS
compilation is checked separately; Apple playback is unsupported there.
