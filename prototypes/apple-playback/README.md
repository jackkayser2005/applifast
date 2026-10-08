# Applifast playback gate

This standalone Windows probe tests MusicKit JS v3 in Evergreen WebView2 before
changing the Rust/egui application. It is not the converted Apple Music client.
Catalog audio has passed a full-track account check; the remaining playback and
resource gates are pending. See the
[evidence report](../../docs/applifast/playback-evidence.md).

## Local developer setup

Use an enrolled Apple Developer Program account. Its developer membership and
the Apple Music subscriber account can be different. In
[Certificates, Identifiers & Profiles](https://developer.apple.com/account/),
register a Media ID with MusicKit enabled, then create a linked Media Services
key. Download the `.p8` key once and keep the Key ID and Team ID.
[Apple's instructions](https://developer.apple.com/help/account/capabilities/create-a-media-identifier-and-private-key/)
describe the required account roles.

Save the key under `.secrets/apple-music/` at the repository root. That directory
and `*.p8` files are ignored. Do not paste secrets into chat or commit them.
Ignoring a file prevents normal Git inclusion; it does not encrypt the file.
Keep your own secure backup of the signing key.

Generate a token locally, replacing the example filename and both IDs:

```powershell
node prototypes/apple-playback/generate-token.cjs --key-file .secrets/apple-music/AuthKey_KEYID12345.p8 --key-id KEYID12345 --team-id TEAMID6789
```

The generator uses Node's built-in crypto, signs ES256 with P-256, and atomically
writes a 30-day JWT to `.secrets/apple-music/developer-token.txt`. It restricts
the token to `https://applifast.invalid`, the probe's local virtual origin. It
does not print tokens or contact a server. Re-run it when the token expires.

## Build and run on Windows

Install the [Evergreen WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
and a Rust MSVC toolchain. From the repository root:

```powershell
$env:CARGO_TARGET_DIR = 'target/apple-host'
cargo +stable run --locked --manifest-path prototypes/apple-playback/Cargo.toml -- --token-file .secrets/apple-music/developer-token.txt
```

Type `show` into the console to reveal the diagnostic window. Click **Sign in
with Apple** and enter the subscriber's account only in Apple's authorization
popup. There is no code to extract from an existing Apple Music browser session.
The developer and returned user tokens go into Windows Credential Manager under
`local.applifast.playback-probe`, separate from Spotifast.

Load library pages and select a song. Rows without a catalog ID remain visible;
that alone does not prove a song is an unmatched upload. Verify the representative
upload independently in the subscriber's library when testing that capability.
Guaranteed cloud-only upload support is a follow-up capability, rather than a
first-version selection gate. Enter a catalog song ID to
test a catalog song followed by the selected library song twice. Listen through
every track completely and test seek. The local occurrence queue sends one
original song descriptor to MusicKit at a time, avoiding bulk-loader ID
deduplication. Unavailable playback stops with an error and retains the queue.
This probe does not yet persist a queue or implement the app's context queue rules.

Closing the diagnostic window hides it; the STA playback host continues running.
Type `hide` or `show` to hide/reopen it. A hidden WebView requests the runtime's
low memory target; showing it restores the normal target. Start playback visibly
before measuring minimized operation, since a hidden cold start stalled in the
account test. Type `{"type":"shutdown"}` to exit.
EOF also exits. Other console commands are JSON, for example:

```json
{"type":"library","next":null}
{"type":"pause"}
{"type":"resume"}
{"type":"seek","seconds":60}
{"type":"volume","value":0.5}
{"type":"shuffle","enabled":true}
{"type":"repeat","mode":2}
{"type":"signOut"}
```

The desktop app also supplies a numeric `id` with every song-library page request.
The library reply echoes it. Only the app's currently pending ID may update its
songs, loading state or read error; older, duplicated and untagged replies are
ignored. The diagnostic console still accepts the untagged example above.
Metadata reads run independently of transport commands and supply a 20-second
abort deadline. Sign-out aborts in-flight API reads and writes and suppresses
their late replies. Playlist writes retain their existing confirmation behavior
without a new deadline, since an interrupted write may have reached Apple.
Library read failures retain loaded songs and the local playback queue, and
do not pause audio. Refresh Songs retries the read.

Repeat modes are 0=off, 1=current occurrence, 2=queue. Sign-out invalidates old
responses, deletes the protected user grant, closes popups and clears the
dedicated browser profile. Check the `profileCleared` result. A durable
sign-out marker prevents interrupted cleanup from restoring an old grant.
The profile lives in `%LOCALAPPDATA%/Applifast/playback-probe`. The Evergreen
runtime manages its own browser storage and updates. This probe creates no
decrypted-media files or hosted backend and adds no application telemetry.
MusicKit contacts Apple's SDK CDN, authentication, API and streaming/license
services, including Apple's normal playback activity reporting. Library metadata
and player counters appear in diagnostic stdout; credentials and raw SDK errors
do not. Keep account-specific diagnostic logs private.

## Resource measurement

Keep the console process running. The host prints its process ID as `hostReady`.
Hide the diagnostic window during each real playback/paused sample. In another
PowerShell terminal, create the output directory and run:

```powershell
New-Item -ItemType Directory -Force .cache/apple-measurements | Out-Null
& prototypes/apple-playback/measure.ps1 -RootProcessId 12345 -State playing -OutputPath .cache/apple-measurements/playing.json
& prototypes/apple-playback/measure.ps1 -RootProcessId 12345 -State paused -OutputPath .cache/apple-measurements/paused.json
```

Replace 12345 with the host PID. Defaults are 120 seconds warm-up and 300 seconds
sampling. The sampler counts the app's process descendants, including browser,
renderer, GPU and helper processes, retaining surviving reparented children and
checking PID reuse. CPU is normalized by all logical processors. It enforces
250 MiB peak aggregate private memory, under 1% average total CPU while playing
and under 0.2% paused. Root exit, incomplete intervals or uncertain counters
prevent a passing result. Polling cannot observe a process born and exited
entirely between snapshots, and state labels do not prove playback. Review
process coverage and verify sound independently. These measurements concern
the isolated engine; final acceptance must include the integrated egui app and
a one-hour gaming/listening session.

## Focused checks

```powershell
node prototypes/apple-playback/generate-token.cjs --self-test
node prototypes/apple-playback/web/test-bridge.cjs
& prototypes/apple-playback/test-measure.ps1
cargo +stable fmt --manifest-path prototypes/apple-playback/Cargo.toml --all --check
cargo +stable clippy --locked --manifest-path prototypes/apple-playback/Cargo.toml --all-targets -- -D warnings
cargo +stable test --locked --manifest-path prototypes/apple-playback/Cargo.toml
cargo +stable test --locked --manifest-path prototypes/apple-playback/Cargo.toml windows::tests::native_store_round_trip -- --ignored --exact
cargo +stable run --locked --manifest-path prototypes/apple-playback/Cargo.toml -- --self-check
```

The ignored test uses an isolated dummy grant and deletes it. The self-check
creates a hidden WebView2 host and reports its runtime version without loading
MusicKit or authenticating. Send shutdown or EOF afterward. Mock bridge tests,
credential storage and runtime initialization do not establish actual audio.
Non-Windows builds report unsupported playback. All ordinary application screens
remain in egui; no production layout or theme changes are part of this probe.

The probe source is MIT. Apple's SDK is loaded directly from Apple and is subject
to Apple's developer/service terms. WebView2 is an installed Microsoft runtime,
not a vendored CDM. Neither third-party runtime is relicensed by this repository.
