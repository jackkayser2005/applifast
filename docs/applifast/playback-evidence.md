# Apple Music playback evidence

Research and account testing date: 2026-10-07. Catalog playback is audible and
complete in the standalone probe. The application conversion remains gated on
the remaining playback checks and resource measurements below.

The user narrowed the first usable version to catalog music and playable synced
iCloud library songs. Guaranteed playback of genuinely cloud-only uploads is
no longer a prerequisite for porting the interface. Such songs remain visible,
retain their library identities, and must report unavailable playback clearly.
The resource budget remains unchanged.

## Route decision

**Native playback cannot be selected from the current evidence.** The strongest
inspected native approaches depend on a separately hosted proprietary Widevine
CDM. This project has no established component-use, acquisition, redistribution,
or update rights for that integration. Google's [Widevine overview](https://developers.google.com/widevine/drm/overview)
states that use requires a license agreement. A downloadable component or one
already installed by a browser does not establish this project's rights. This
is an unmet project selection gate, not a legal verdict about those projects.
No CDM was downloaded or loaded and no protection integration was prototyped.

**MusicKit JS v3 in Evergreen WebView2 is the fallback candidate, not yet an
accepted playback engine.** Apple documents browser playback through
[MusicKit on the Web](https://developer.apple.com/musickit/), alongside Apple
platform and Android integrations; that page supplies no native Windows SDK.
Browser support does not by itself prove embedded WebView2 compatibility,
library playback, or the gaming budget. Successful tests must precede
the engine selection and substantial application conversion.

## Pinned source findings

| Project | Inspected revision | Source license | Relevant result |
| --- | --- | --- | --- |
| Sonora | `9f6567874582749b675a59ee705a8447191da6b6` (2026-10-02) | GPL-3.0-or-later in manifest | Hosts a genuine CDM separately from its native audio path, but drops library uploads with no catalog ID. |
| Kopuz | `defc02fc0c7c98067d31376158fb0ac8cebd8709` (2026-10-06) | EUPL-1.2 in manifest | Distinguishes cloud-library dispatch and uploaded assets from catalog encodes; this is source evidence, not account validation. |

- Sonora's [library conversion and regression test](https://github.com/sonorahq/sonora/blob/9f6567874582749b675a59ee705a8447191da6b6/crates/music/src/apple/wire.rs)
  exclude uploads without a catalog counterpart. Its [stream path](https://github.com/sonorahq/sonora/blob/9f6567874582749b675a59ee705a8447191da6b6/crates/music/src/apple/stream.rs)
  uses Apple's web-playback and license endpoints with a CDM challenge, and
  [component acquisition](https://github.com/sonorahq/sonora/blob/9f6567874582749b675a59ee705a8447191da6b6/crates/widevine/src/fetch.rs)
  uses Google's component update service. Its [authentication](https://github.com/sonorahq/sonora/blob/9f6567874582749b675a59ee705a8447191da6b6/crates/music/src/apple/auth.rs)
  reads Apple's web-player bearer token, rather than requiring this app's own
  developer token. These choices do not meet this fork's requirements.
- Kopuz's [stream path](https://github.com/Kopuz-org/kopuz/blob/defc02fc0c7c98067d31376158fb0ac8cebd8709/crates/server/src/applemusic/stream.rs)
  dispatches retained library IDs separately and handles an uploaded asset as
  either a plain file or encrypted media. It explicitly rejects one HLS response
  shape. Its current path also includes decrypted-media caching and verbose
  license-response logging, both excluded here. Its [authentication](https://github.com/Kopuz-org/kopuz/blob/defc02fc0c7c98067d31376158fb0ac8cebd8709/crates/server/src/applemusic/auth.rs)
  scrapes a web-player bearer, and [CDM discovery](https://github.com/Kopuz-org/kopuz/blob/defc02fc0c7c98067d31376158fb0ac8cebd8709/crates/server/src/applemusic/widevine/discover.rs)
  can use a browser-installed component. Neither establishes Applifast's
  standalone integration rights.
- Source licenses were checked in the pinned [Sonora manifest](https://github.com/sonorahq/sonora/blob/9f6567874582749b675a59ee705a8447191da6b6/Cargo.toml)
  and [Kopuz manifest](https://github.com/Kopuz-org/kopuz/blob/defc02fc0c7c98067d31376158fb0ac8cebd8709/Cargo.toml).
  No implementation code from either project was copied into this MIT fork.

The web-playback and license endpoint names are observations from those sources,
not a claim that Apple documents them for independent native use. No request was
made to those endpoints in this research cycle. Another project's token must
not substitute for our own credentials.

## Requirements for the fallback prototype

- Supply this app's own developer token. Apple's [token instructions](https://developer.apple.com/documentation/applemusicapi/generating-developer-tokens)
  require ES256 signing with a MusicKit private key and limit expiration to six
  months. Keep signing keys outside the repository; import the token locally.
  Apple authorization yields the user grant; do not ask for an iCloud password
  in Rust or expose authorization responses in logs.
- Retain library resource ID, optional catalog ID, and Apple playback parameters
  independently. A missing catalog ID never removes an upload or causes a
  name-based catalog substitute. Drive playback through MusicKit's supported
  item parameters and test playable library songs on the user's account.
  Record genuine cloud-only upload support separately when provenance is known.
- Keep all COM objects on a dedicated STA thread with a message pump; egui and
  runtime workers communicate through channels. Microsoft's [threading model](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/threading-model)
  prohibits cross-thread WebView2 access and describes popup deferrals.
- Own the playback host independently from the visible window. Use a dedicated
  profile, explicit Apple authorization popups, origin-checked bridge messages,
  and profile/authentication clearing on sign-out. Evergreen supplies runtime
  servicing; do not separately discover or load browser CDM binaries.
- Count the complete [WebView2 process group](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model),
  including browser, renderer, GPU, audio and other helpers, plus the native app.

## Acceptance ledger

### Real-account progress

The locally generated developer token passed Apple's `/v1/test` with HTTP 200.
The user completed the Apple authorization popup, and the resulting grant was
saved in Windows Credential Manager. Restarting the probe restored authorization
without another password prompt. The loaded official v3 SDK reported
`3.2526.0-prerelease.x`; its DRM-capability and secure-context checks were true.
Capability detection alone is not a playback result.

A catalog song played through its full 169-second duration, emitted its ended
state and repeated successfully. The user confirmed audible output through the
normal device. A 100-song library page loaded with a next-page path. Four rows
had no playback parameters or catalog ID and remained visible. A library song
with no catalog ID in that response subsequently played using its original
library identifier and reported a full 290-second duration. The user has not
confirmed its upload/match provenance, so this does not establish genuinely
cloud-only upload or matched-song support.

The first resource sample includes the temporary controller process and all
of its descendants. The diagnostic window was reopened when the user needed
to inspect the library, so this run cannot qualify as the minimized acceptance
sample. It measured 302.166 seconds after the two-minute warm-up: peak aggregate
private memory 342.496 MiB and average total CPU 0.330944%. Memory exceeded the
250 MiB limit, and sampling gaps made accounting incomplete. These numbers do
not pass acceptance. Account diagnostics were ignored and removed after the
results were recorded; no account metadata is included in this report.

The sampler now reads counters only for possible process-group members; its
previous system-wide counter reads could exceed the sampling interval. The host
now marks the WebView hidden and requests the runtime's low memory target while
hidden or minimized, restoring the normal target when shown. This follows
[Microsoft's performance guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance)
and keeps playback running rather than suspending scripts. A 3.336-second live
sampler check had no accounting gaps and peaked at 236.562 MiB, but its short
duration does not satisfy acceptance. Clean playing and paused samples remain
required. A hidden cold start stalled before playback began; showing the window
allowed playback to start. The minimized sample begins after visible startup.

A first minimized mixed-queue attempt finished the catalog occurrence but
failed on a library row with no playback parameters. The selected occurrence
and the rest of the queue remained intact. That sample was stopped and cannot
qualify as continuous playback. Selecting a library row with Apple's playback
parameters succeeded, and an explicit Next followed by a seek to 30 seconds
produced advancing playback at that position. A fresh minimized sample used
that playable library row, catalog occurrences and queue repeat. Hidden
automatic transitions from library to catalog and through duplicate occurrences
succeeded, with no additional errors during the sample.

The clean minimized playing sample used a 120-second warm-up and 300.332-second
measurement on this Windows PC. Accounting was complete across 12 associated
processes, including the temporary Node controller and all WebView2 helpers.
CPU percentages are normalized across this PC's 16 logical processors.
Average total CPU was **0.131690%**, below 1%. Peak aggregate private memory was
**285.406 MiB**, above 250 MiB; all 290 snapshots exceeded the limit (minimum
277.301 MiB). This is a failed memory gate, not a transient single-sample spike.

The minimized paused sample used another 120-second warm-up and 300.140-second
measurement. Native state reports confirmed paused playback with an unchanged
47-second position. Accounting was complete. Average total CPU was
**0.008134%**, below 0.2%, while peak aggregate private memory was
**278.297 MiB**, above 250 MiB. Both CPU gates passed in the standalone probe;
the memory gate failed in both states. Both samples used Evergreen WebView2
154.0.4258.62, MusicKit 3.2526.0-prerelease.x and Rust 1.97.1. These debug/MSVC
probe samples include
the temporary controller; final measurements must include the integrated egui
application and its helpers. No production footprint is inferred by subtracting
the controller. The one-hour gaming soak has not been run.

The focused route-selection cycle therefore ends without a selected engine:
native component rights are unresolved and WebView2 exceeds the memory budget.
The listening slice is incomplete. Further memory experiments need a concrete
new hypothesis; repeating the same sample cannot change that decision.

Apple's successful popup close initially emitted the same warning as a manual
cancel. The host now distinguishes SDK dismissal from the user closing the
popup. The existing Windows-focused tests and strict Clippy still pass; another
real cancellation/reauthorization cycle remains to be exercised.

### Automated and startup checks

The standalone probe in `prototypes/apple-playback` now compiles on Windows.
It owns COM/WebView2 on a dedicated STA thread and keeps the host alive while
its diagnostic window is hidden. No production app engine has been changed.
The Windows startup/shutdown self-check initialized Evergreen
**154.0.4258.62** without loading MusicKit or using real credentials. The isolated
Windows Credential Manager dummy-grant write/read/delete check passed.

The probe uses `webview2-com 0.39.1`, `windows 0.62.2`, `keyring-core 1.0.0`,
`windows-native-keyring-store 1.1.0`, serde/serde_json, base64 and url. Versions
are recorded in its separate `Cargo.lock`. Probe source is MIT; the Microsoft
runtime and Apple SDK retain their vendor terms. No third-party playback source
was imported. Evergreen supplies browser component updates. MusicKit JS is
loaded from Apple's v3 CDN rather than copied into the source tree.

The SDK downloaded for source inspection on this date had SHA-256
`21908b72bdcdeea3ec91fa40b105d0b9a2ba07c2a3fe32ea77edcd26e735012c`.
That hash identifies the inspected asset, not future runtime downloads. Its bulk
resource loader indexes descriptors by song ID. The probe therefore maintains
local queue occurrences and passes one original catalog/library descriptor at
a time; a cloud-only library ID is never replaced by a catalog counterpart.
Uploaded audio and expanded error/recovery cases still require account tests.

Windows probe formatting, strict Clippy and six Rust tests passed. Node checks
verified a generated JWT against an ephemeral public key, identifier and
duplicate-occurrence preservation, unavailable-song retention, page validation,
failed-playback queue retention and stale authorization rejection. PowerShell
checks covered process-tree selection, PID reuse and CPU normalization. Short
live sampler runs verified helper discovery and correctly failed the acceptance
duration. None of these are full-track playback or budget results.

Root formatting passed with Rust 1.98. The unchanged root's default Clippy check
was blocked by `projectm-sys`: `VCPKG_INSTALLATION_ROOT` is unset. Required
all-feature checks use the same dependency. Packaging checks also encounter
missing Ruby and incompatible Windows/Unix shell paths. Ruby/Bundler/Jekyll and
Linux/macOS Rust targets are absent; those platforms and site build have not
been tested. Do not treat the root contribution suite as passing.

There are no production layout, colors or wording changes, so no main-window
before/after visual comparison is claimed. The temporary diagnostic HTML is
for authorization and playback investigation. App integration, production
visual evidence, restoration, media controls, taskbar/tray and mini-player
recreation remain behind the selection gate.

| Gate | Native | WebView2/MusicKit |
| --- | --- | --- |
| MIT-compatible implementation and established component rights | Blocked as above | Runtime integration and dependencies require verification |
| Own developer token and subscriber authorization | Not exercised | Passed token verification, real authorization and credential restoration |
| Complete catalog and playable synced library tracks | Not exercised; Sonora upload path fails statically | Catalog complete and audible; library playback observed, completion pending |
| Genuine cloud-only uploads (follow-up capability) | Not exercised | Provenance unconfirmed; unsupported songs retained |
| Seek, mixed queues and duplicate occurrences | Not exercised | Next, seek to 30 seconds, hidden automatic library/catalog transitions, duplicate occurrences and queue repeat exercised successfully |
| Tray/minimized playback, main/mini-player recreation | Not exercised | Hidden probe playback and transitions passed; production tray and window recreation pending |
| Aggregate private memory at most 250 MiB | No measurement | Failed: playing 285.406 MiB peak, paused 278.297 MiB peak |
| Minimized playing average total CPU under 1% | No measurement | Passed standalone sample: 0.131690%, complete accounting, 300.332 seconds |
| Minimized paused average total CPU under 0.2% | No measurement | Passed standalone sample: 0.008134%, complete accounting, 300.140 seconds |
| One-hour gaming/listening soak and error recovery | Not exercised | Not exercised |

Performance samples require a two-minute warm-up followed by five minutes per
state on the user's Windows PC. Record runtime/build versions, track kinds,
process IDs, total CPU normalization, memory samples and pass/fail results.
Failure cannot silently relax a threshold. Mock data, compile success and an
initialized WebView are not playback evidence. No engine is selected until all
applicable gates pass; retain actionable blockers if neither passes.
