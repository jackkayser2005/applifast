# Windows private tester guide

This is a development preview for invited Windows x64 testers. Public release
distribution, installers and clean-machine acceptance are still pending. A preview
can report the inherited version `spotifast 0.12.0`; use its `BUILD.txt` source
revision when reporting a problem. The original Spotifast downloads are a
different product.

## Open shared music

Paste an Apple Music song, album, artist or playlist share URL into Search and
press Enter. This opens its page without changing playback. A song opens its
album when Apple supplies one. You can also launch `Applifast.exe` with a quoted
link; it forwards to an existing instance or waits for sign-in. To start the
linked music explicitly, run `Applifast.exe play-uri "https://music.apple.com/…"`.
Use a complete share URL, not the shortened example here. The signed-in account's
storefront determines availability. Shortened URLs and registering a Windows
default URL handler are not supported yet.

## Start listening

1. Get the portable Applifast preview and its checksum from the maintainer.
   Extract the whole ZIP. Quit an older build through its tray menu before
   starting `Applifast.exe`. Closing its window can leave music playing.
2. Install or repair the [Evergreen WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
   if the app reports a host initialization failure. An installed Edge browser
   alone is not the production runtime. See [Microsoft's distribution guidance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution).
3. Keep the included **developer-token.txt** beside **Applifast.exe**. It
   authenticates the app; it contains a signed MusicKit JWT, with no listener
   credentials or private signing key. It loads automatically when no valid
   app token is already stored on this PC.
   Testers do not need their own Developer Program membership, signing key,
   Node installation or Rust toolchain. The maintainer creates the developer
   token; each tester authorizes their own Apple Music subscriber account.
   See [Apple's developer-token documentation](https://developer.apple.com/documentation/applemusicapi/generating-developer-tokens).
4. Click **Sign in with Apple** when it becomes available. Enter account
   credentials only in Apple's authorization popup. There is no code to extract
   from an existing browser session. MusicKit supplies the user authorization
   token; never copy browser cookies or user tokens. See [Apple's authentication
   documentation](https://developer.apple.com/documentation/applemusicapi/user-authentication-for-musickit).
5. Open Songs, choose a track and check your audio output. Loaded library
   metadata and the local queue survive restart; restored playback starts
   paused. These caches contain no downloaded audio and do not provide offline
   listening. A song without usable Apple playback parameters stays visible
   with an error. It is never replaced with another recording.

Windows checkpoint replacement can now complete while another reader permits
delete sharing. A read-only file or a reader that denies deletion can still
block a save; quit software holding that file and retry. This is metadata and
session persistence only. See the [reproduction and limits](windows-save-evidence.md).

If the app token is missing or expired, download a fresh preview from the
maintainer and extract all its files. `BUILD.txt` records the bundled token's
expiry. There is no token-renewal server; these builds currently use 30-day
tokens. An expired stored token automatically falls back to a valid companion
file. A valid manually imported token takes precedence. Advanced recovery still
offers **Import developer token** on errors and under Settings > Account.
Leave its path empty to open the file chooser, or supply a quoted Windows path.
Never select a `.p8` key or paste a JWT into the path field. A source checkout
without the companion file still needs this manual setup.

Import checks run off the interface thread. Cancelling the file chooser or
rejecting an invalid or expired file preserves the current account, library,
queue and playback. Duplicate imports are disabled while selection/validation
is pending. Successful validation restarts the playback host to use the new
token. Apple still verifies its signature and access. Local validation does
not prove that a developer key is unrevoked or that Apple will accept it.

Open **Settings > Keyboard shortcuts** or press **Ctrl+/**. **Alt+1** through
**Alt+5** open Home, Songs, Favorites, Albums and Artists. **Ctrl+M** opens the
mini player, **Ctrl+Shift+Q** opens the queue, and **F5** refreshes music pages.
The [current capabilities and pending checks](listening-slice.md) distinguish
implemented controls from verified real-account behavior.

Song and collection menus show supported Apple actions. Known favorites show
filled hearts in song rows and the player bar. These are read-only indicators;
playlist, album and search reads also request Apple's explicit song favorite
flags. Unknown songs stay unmarked. Favorites are readable,
but changing favorites, artist follow, radio, playlist removal/reordering and
editing playlist details remain unavailable in this preview. Use Apple Music
for those changes, then refresh here. Playlist creation and adding songs remain
available. Cut, Paste and Delete do not edit Apple playlists; Copy still works.

**Copy link** gives catalog items a public `music.apple.com` URL in your account's
storefront. Library items retain an internal `apple:` link with their exact ID;
these links require access to that library and are not public sharing links.
**Open in Apple Music** opens the browser for catalog items only. Uploaded songs
are never substituted with catalog matches to create a link. Connect/device and
favorite-write commands are absent from CLI help and return an unsupported error
if invoked explicitly.

## If setup fails

| What you see | What to do |
| --- | --- |
| Cannot open/read developer token file | Choose a readable local `.txt` file. Do not enter the token itself in the path field. |
| Expected a signed MusicKit developer JWT | Ask the maintainer for the signed token file, not the private `.p8` key. |
| Developer token must be UTF-8 / exceeds 32 KiB | Use the original token file rather than an HTML download or key export. |
| Developer token expired | Obtain a fresh token and import it from Account settings. The local generator currently issues 30-day tokens. |
| Invalid token dates or origin | Check Windows date/time. Ask the maintainer to regenerate for `https://applifast.invalid`. |
| WebView2 host failed | Quit other Applifast builds, install/repair Evergreen, then reopen. Report a fixed host error or HRESULT without credentials. |
| Authorization cancelled or rejected | Cancel the pending authorization and retry Sign in with Apple. Check the intended subscriber account and Apple Music access. |
| Apple rejects requests after sign-in | Refresh/retry. Persistent authorization failures may require reauthorization or developer-token renewal. They do not always mean an expired subscription. |
| Empty Favorites or Home feed | Refresh the relevant music page. Favorites shows loaded songs Apple marks as favorites. A real empty result and a failed request have different states. |

Do not delete profile folders or Credential Manager entries while music is
running. Use the app's **Sign out** action when you intend to clear the account.
Signing out invalidates pending work and requests browser authentication-data
clearing. Fresh-machine sign-in, expiry and profile-clearing acceptance still
need a separate real Windows account test; deterministic demos are not proof.

## Local data and privacy

| Data | Windows location |
| --- | --- |
| Preferences | `%APPDATA%\paolino\applifast\config\settings.json` |
| Queue/library snapshot | `%LOCALAPPDATA%\paolino\applifast\data\apple-session.json` |
| Other durable state and current log | `%LOCALAPPDATA%\paolino\applifast\data\` (`applifast.log`, `panic.log`) |
| Main-window geometry and interface memory | `%LOCALAPPDATA%\paolino\applifast\data\window.ron` |
| Artwork and disposable caches | `%LOCALAPPDATA%\paolino\applifast\cache\` |
| Isolated playback browser profile | `%LOCALAPPDATA%\Applifast\playback-probe\` |
| Developer/user tokens | Windows Credential Manager, service `local.applifast.playback-probe` |

Older previews wrote `spotifast.log` in the same Applifast data directory and
used the upstream window-memory path. New builds leave those files alone and
start with independent main-window geometry and zoom. Settings, Apple grants,
the library/queue snapshot and mini-player preferences retain their existing
paths. If a report concerns an older preview, include its log and source revision.

The selected file is read locally, bounded and validated again by the playback
host before credential storage. `.p8` paths are rejected before opening.
Token contents do not cross interface events or appear in diagnostic messages.
Import does not delete the original token file. Store it privately; ignoring a
file in Git does not encrypt it. Signing keys remain only with the maintainer.

MusicKit, authorization, catalog/library requests, artwork and streamed audio
contact Apple. The optional Lyrics view sends song artist/title/album/duration
to LRCLIB. Applifast adds no hosted backend, app telemetry, DRM bypass or
decrypted-media cache. See the listening-slice document for network details.

## Report a useful test result

Include the source revision from `BUILD.txt`, Windows version, reproduction
steps, expected/actual result and whether main, mini-player or tray controls
were involved. A screenshot can help, with account names and private paths
redacted. Never include passwords, JWTs, `.p8` contents, user tokens, cookies,
or raw authorization responses.

Before widening the beta, record clean-machine setup, full-track listening,
seek, network recovery, output changes, restart restoration, tray/main/mini
recreation, and an hour of listening while gaming. Aggregate WebView2/helper
memory and CPU separately from executable size. Existing local tests and
read-only account diagnostics do not complete those gates.
