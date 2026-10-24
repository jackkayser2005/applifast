---
title: Download
description: Windows preview installation, checksums, updates and release availability.
nav_order: 1
---

## Windows preview

Applifast is currently an invited **Windows x64 development preview**.
Ask the maintainer for the current installer or portable ZIP and its SHA-256
checksum. This documentation change does not publish a public beta download.

Original Spotifast releases, Homebrew, AUR and Flatpak packages install the
upstream Spotify product, not this Windows Apple Music port.

## Install

Quit older Applifast builds through their tray menus. Run the preview installer,
then open **Applifast** from Start. It installs per user in
`%LOCALAPPDATA%\Programs\Applifast`, without an administrator account.
A desktop shortcut is optional.

The portable ZIP contains the app, companion developer token, build record,
tester guide and licenses. Extract the whole archive. Keep **Applifast.exe**
and **developer-token.txt** together.

You need an Apple Music subscription, internet access and the
[Evergreen WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).
The installer does not download that runtime.
[Getting Started](/getting-started/) explains authorization and recovery.

## Check your download

Compare the file with its supplied SHA-256 checksum:

```powershell
Get-FileHash -Algorithm SHA256 -LiteralPath .\applifast-preview-SOURCE-setup.exe
```

Replace the example filename with your download's name. Current previews are
not Authenticode-signed. A matching hash checks file integrity, not publisher
identity.

**BUILD.txt** records the source revision, executable hash, preview status and
app-token expiry. Include these details in reports; a Windows file version
alone does not distinguish development snapshots.

## Update or remove

Apple mode does not use the upstream Spotifast updater. Quit through the tray,
then install a replacement preview manually. Keep the app and token together.
A valid stored developer token takes precedence until renewal is needed.

Uninstall through Windows Settings. Uninstalling does not sign out or erase
settings, account metadata or the browser profile. Use **Sign out** inside
Applifast before uninstalling if you want to clear account authorization.

## Other platforms and public releases

Linux and macOS compilation is checked, but Apple playback is unsupported.
The private package qualifies Windows x64, not a Windows ARM installer.

A public beta still needs final runtime acceptance, release metadata, a signing
decision, verified artifacts and download links. Follow
[the fork](https://github.com/jackkayser2005/applifast) and its
[release checklist](/applifast/release-readiness.html).
