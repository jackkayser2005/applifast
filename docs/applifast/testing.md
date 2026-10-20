# Windows private tester guide

This is a development preview for invited Windows x64 testers. Public release
distribution and clean-machine acceptance are still pending. Current command
help and `--version` identify `applifast`; older previews can report
`spotifast 0.12.0`. Use the `BUILD.txt` source revision and file version when
reporting a problem. The original Spotifast downloads are a different product.

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

1. Get the Applifast preview installer or portable ZIP and its checksum from the
   maintainer. Quit older builds through their tray menu first. Closing a window
   can leave music playing. Run the installer and open Applifast from Start, or
   extract the whole ZIP and run `Applifast.exe`. These preview executables are
   unsigned; checksums detect changed downloads and do not prove publisher identity.
   The installer needs no administrator account. It installs the app and token
   together in `%LOCALAPPDATA%\Programs\Applifast`, adds a Start entry and offers
   an optional desktop shortcut. It does not download WebView2 or register link
   handlers. Updates require quitting the old build and running the new installer.
2. The app needs the [Evergreen WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).
   If host initialization fails, first quit other Applifast builds through their
   tray menus and click **Retry connection**. This restarts the host using saved
   credentials or the companion developer token, without restarting the app.
   Once connected, **Sign in with Apple** opens authorization when needed.
   WebView2 `0x8007139F` can indicate incompatible options
   in a shared browser process. An installed Edge browser alone is not the
   production runtime; install or repair Evergreen if it is missing or still
   fails after other builds exit. See [Microsoft's distribution guidance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution)
   and [environment errors](https://learn.microsoft.com/microsoft-edge/webview2/reference/win32/webview2-idl).
   After the local playback page loads, the app waits up to 45 seconds for Apple's
   SDK. A missing SDK produces the existing connection error and enables Retry
   connection. Check your network, then retry. A late SDK arrival cannot finish
   that failed attempt; retry creates a fresh host. Native environment/controller
   creation, profile clearing and bootstrap registration now share their own
   45-second completion deadline. A timeout offers Retry connection and preserves
   saved setup. This does not time-limit Apple's authorization popup or guarantee
   a total startup duration. See [native setup checks](native-setup-deadline.md).
3. Keep the included **developer-token.txt** beside **Applifast.exe**. It
   authenticates the app; it contains a signed MusicKit JWT, with no listener
   credentials or private signing key. It loads automatically when no valid
   app token is already stored on this PC. The installer puts them together.
   Testers do not need their own Developer Program membership, signing key,
   Node installation or Rust toolchain. The maintainer creates the developer
   token; each tester authorizes their own Apple Music subscriber account.
   See [Apple's developer-token documentation](https://developer.apple.com/documentation/applemusicapi/generating-developer-tokens).
4. Click **Sign in with Apple** when it becomes available. Enter account
   credentials only in Apple's authorization popup. There is no code to extract
   from an existing browser session. MusicKit supplies the user authorization
   token; never copy browser cookies or user tokens. See [Apple's authentication
   documentation](https://developer.apple.com/documentation/applemusicapi/user-authentication-for-musickit).
   Use **Cancel** on the sign-in card if you change your mind. Cancellation now
   releases a pending authorization even if its old SDK promise never settles.
   Closing the native popup cancels that attempt too, allowing another sign-in.
   Include close, Cancel and retry in the fresh-PC checklist; deterministic
   cancellation tests do not replace checking Apple's real popup.
5. Open Songs, choose a track and check your audio output. Loaded library
   metadata and the local queue survive restart; restored playback starts
   paused. These caches contain no downloaded audio and do not provide offline
   listening. A song without usable Apple playback parameters stays visible
   with an error. It is never replaced with another recording.

Windows checkpoint replacement can now complete while another reader permits
delete sharing. A read-only file or a reader that denies deletion can still
block a save; quit software holding that file and retry. This is metadata and
session persistence only. See the [reproduction and limits](windows-save-evidence.md).

If the app token is missing or expired, install a fresh preview from the
maintainer or extract its complete ZIP. `BUILD.txt` records the bundled token's
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
**B** adds the playing song to Favorites in either window. It does nothing for
an existing or pending favorite, while typing, or while a dialog is open.
The [current capabilities and pending checks](listening-slice.md) distinguish
implemented controls from verified real-account behavior.

After playing a song with artwork, open its lyrics or Now Playing view, switch
to the mini player and back, then close to the tray and reopen. Check that the
cover and backdrop match the song while music and the queue continue. Recreated
windows now discard stale artwork handles and old pending image results; they
can load the images again from the artwork cache. The automated lifetime check
does not replace this desktop trial. See [window artwork evidence](window-artwork.md).

To choose what follows the playing song, right-click a song, selected songs or
an album and choose **Play next**. It goes before manual additions; **Add to
queue** appends after them. Album choices wait for every page before appearing.

To keep a library playlist at the top of the sidebar, open the playlist, click
**More** and choose **Pin to top**. The menu changes to **Unpin** while it is pinned.
Right-clicking the playlist in the sidebar offers the same controls. Drag within
the pinned block to choose its order. Pins are saved locally in settings, persist
after Quit/reopen and do not synchronize to Apple's apps or change playlist songs.
The page menu offers pinning only when that exact playlist is in the loaded
library. See [playlist pin checks](playlist-pins.md).

If Songs cannot refresh, cached rows stay available and the error row offers
Retry. Rejected app credentials require a fresh developer token imported in
Settings > Account. Rejected account authorization points to checking your
subscription and signing out/in there. Rate-limit and service errors ask you
to wait before retrying. No error signs you out or repeats playlist/Favorites
writes automatically. Include real rejection and recovery in the account-lifecycle
checklist; simulated errors do not establish revoked-credential acceptance.
Each metadata read has a 20-second deadline. It releases the app's local wait
even if MusicKit ignores abort; a late response cannot replace the error with
stale data. Favorites confirmation reads use the same deadline and leave unknown
results unconfirmed. Refresh before retrying a write. Signing out releases pending
API waits locally, but cannot undo a write Apple may already have accepted.
See [the deterministic cancellation checks](read-cancellation.md).

For queue removal, right-click an upcoming song and choose **Remove from queue**.
In the mini-player playlist, Ctrl-click rows, then choose **REM > Remove selected**.
The currently playing row is excluded. Queue two copies of a song and remove
just one; confirm the other remains and the current song keeps playing. Quit
and reopen to check restoration. Removal affects this local listening queue,
including its repeat context, and never changes your Apple library or playlists.
The mini-player's **Remove all** keeps context songs and clears manual additions.

Windows Settings > Apps can uninstall the installed preview. Uninstall removes
the app files and shortcuts and preserves your Applifast settings, library/queue
cache, WebView2 profile and Windows Credential Manager entries. Sign out in the
app before uninstalling if you want its account authorization and snapshot cleared.
Portable ZIPs keep their existing manual folder setup.

If sign-out reports that Apple browser data could not be cleared, quit Applifast
through its tray menu and reopen it before signing in again. The playback host
stops on that failure; startup retries cleanup using the saved sign-out marker.
This error does not confirm that browser cleanup finished.

Song and collection menus show supported Apple actions. Known favorites show
filled hearts in song rows and the player bar. Hollow hearts and song/selection
menus offer Add to Favorites (up to 100 distinct non-favorite songs per action).
The mini-player uses the inactive Apple EQ slot for the current song's heart.
Its title-bar right-click menu also offers Add to Favorites, including when
rolled up. Filled hearts remain read-only in both window modes.
Hearts fill immediately. Exact confirmation keeps successful members and restores
unconfirmed members to their previous state with a refresh-before-retry message.
Refresh before retrying: an interrupted write may have reached Apple. Pending
hearts are not saved as confirmed favorites on restart. Live-account additions
still need testing; removing a favorite requires Apple Music and Refresh here.
Playlist, album and search reads also request Apple's explicit song favorite
flags. Unknown songs show a hollow add button. Artist follow, radio, playlist removal/reordering and
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
| WebView2 could not start with the current browser profile | Quit other Applifast builds through their tray menus, then click Retry connection. The app keeps your saved setup. |
| WebView2 host failed | Check/install Evergreen, then click Retry connection. Report the fixed host error or HRESULT without credentials. |
| WebView2 setup timed out | Check the Evergreen runtime and retry the connection. Saved setup is retained. Include the build revision and fixed error if it repeats. |
| Apple playback page could not load | Click Retry connection. If it persists, report the app revision and this fixed error, without credentials or browser-profile files. |
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
