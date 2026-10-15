# Applifast Windows beta readiness

Snapshot: October 14, 2026. This is the release checklist, not a release claim.
The `a15ba42` baseline (PR #53) fixes pending-authorization cancellation
on top of paused restoration, full-track regressions, the SDK startup deadline
and Favorites shortcut.
The `fix/reduce-motion-controls` follow-up completes the shared setting's switch,
artwork-tint and drag-feedback paths. Its deterministic checks and native resting
comparisons are documented [separately](reduce-motion-controls.md); fresh-PC
authorization and interactive motion acceptance stay open.
The `feat/apple-queue-removal` follow-up adds exact upcoming-row removal in the
main and mini-player queues, protected from stale actions. Its isolated native
test passes while muted playback continues; see [queue evidence](queue-removal.md).
The `feat/apple-play-next` follow-up adds front insertion for songs, selections
and complete albums, with chronological handling of delayed album responses.
See [Play next evidence](play-next.md).
The `fix/apple-api-recovery` follow-up preserves safe API failure status and
shows recovery guidance in Songs without losing cached rows or playback.
Its read-only fresh-profile probe passes library/catalog reads; earlier isolated
rejections remain unexplained. See [API recovery evidence](api-recovery.md).
The `fix/window-artwork-lifetime` follow-up resets softened covers and the lyrics
backdrop when attaching a recreated window, releasing stale texture ownership
and rejecting old pending image results. Its regression reproduces the retained
old manager before the fix and passes afterward. See [window artwork evidence](window-artwork.md).
The `feat/apple-playlist-pins` follow-up exposes the existing local pin/unpin
action in a loaded library playlist's More menu. Pointer, ordering, refresh and
settings-reload regressions pass. See [playlist pins](playlist-pins.md).
`main` remains at
`995c768`; these preview changes have not
been merged into a public release.
The `release/apple-windows-integration` branch assembles the feature stack for
review against that `main` baseline. See the [source and PR #13 trace](integration.md)
for scope, evidence and the remaining integration gates.

## Next private trial: Dad's PC

The next handoff is an invited Windows x64 preview. A tester needs an Apple
Music subscription, network access and WebView2 Evergreen. The package supplies
the app developer token; the tester authorizes their own Apple account. They
do not need Developer Program membership, a signing key, Rust, Node, or commands.
Use the [Windows tester guide](testing.md).

- [x] Build and verify the optimized `abd1295` installer/ZIP, checksums,
  six-file payload, source manifest and default-feature CLI help. Isolated
  dummy-token packaging and install/upgrade/uninstall fixtures pass.
- [x] Complete the October 12 candidate [credential/package audit](private-preview-audit.md):
  fetched branch coverage, historical secret paths, private-key/JWT patterns,
  original-checkout ignore rules and the intended companion-token exception.
  Repeat against the final release source and payload before publication.
- [x] Pass the isolated native [idempotent Favorites account check](favorites-write-evidence.md):
  restore protected authorization, read library/catalog metadata and re-add
  separately confirmed favorites without audio. New-favorite writes and
  fresh-PC authorization remain unchecked.
- [x] Demonstrate muted native playback, seek and pause with restored protected
  authorization.
- [x] Resolve the native paused-restoration timeout and verify actual saved-position
  resume with both queue occurrences retained. The corrected native check passes
  in 44.86 seconds; see [state-conversion evidence](paused-restoration.md).
- [x] Run the isolated muted full-track check: a complete catalog track, a
  complete catalog-linked library track, then the separate context duplicate
  starts and pauses. Both 90.013-second tracks finished naturally without seeks;
  the native coordinator preserved manual-before-context order.
- [x] Reproduce and fix cancellation blocked by an unresolved authorization
  promise. Deterministic close/cancel/retry and late-result regressions pass.
  The updated native host passes restored-account playback/seek/resume in
  43.80 seconds; actual Apple popup cancellation remains in the fresh-PC check.
- [x] Verify removal of one duplicate during muted native playback, with the
  current position and other occurrence retained. The extended account test
  passes in 45.43 seconds; fresh-PC UI use and audible listening remain open.
- [x] Verify every required hosted CI job on `abd1295` (PR #59): all seven
  [candidate jobs](https://github.com/jackkayser2005/applifast/actions/runs/38489782963)
  pass, including quality, docs, Windows x64/ARM, Linux, macOS and Nix. Repeat
  on the final release commit after integration or further changes.
- [x] Verify the optimized `7dd1650` playlist-pin installer/ZIP, source/payload,
  checksums, dummy-token packaging and isolated install/upgrade/uninstall fixtures.
  The app never launches in those installer fixtures. Local refs/reflogs and ZIP
  pattern checks find no private keys or developer JWTs outside the declared
  companion token, within the [audit limits](private-preview-audit.md).
- [x] Verify the playlist-pin follow-up's own hosted CI: all seven jobs pass on
  `7dd1650` in [run 38625838481](https://github.com/jackkayser2005/applifast/actions/runs/38625838481).
  Repeat required checks on the final integrated release source.
- [ ] On Dad's PC, install through the wizard, open from Start, authorize in
  Apple's popup and load his library without copying another user's profile,
  user token or account cache. Check cancel and retry before successful sign-in.
  The maintainer confirmed that Dad has not tried the preview yet.
- [ ] Listen to a complete catalog song and a synced library song. Seek,
  change songs, add repeated queue entries and try shuffle/repeat.
- [ ] Close to the tray, use media keys, switch between the main and mini-player,
  then Quit and reopen. Confirm the exact queue returns and playback starts paused.
- [ ] If testing writes, add a previously non-favorite song and create a small
  test playlist. Confirm the result in Apple Music, refresh and restart.
- [ ] Record the source revision from BUILD.txt, Windows version, outcome and
  reproducible errors. Do not collect account credentials or authorization logs.

A short private trial can discover onboarding problems. It does not establish
public release readiness. The current previews are unsigned and their bundled
app token expires November 6, 2026 at 17:47 UTC. Renew before expiry and ship a
fresh preview. There is no renewal server or automatic updater.

## Order of the remaining passes

Allow one or two focused testing/fix passes for the first family trial, provided
fresh-account sign-in does not reveal a larger issue. That is an estimate, not
a completion promise. A verified installer already exists; the most useful next
evidence comes from a different PC and account.

1. **Private handoff:** candidate CI passes. Try the installer and the
   sign-in/cancel/retry steps on Dad's PC. Fix anything that blocks opening,
   authorization, loading his library or audible playback.
2. **Daily-use trial:** finish the short listening/queue/tray/restart checklist
   above and check any Favorites/playlist actions he uses. Record failures and
   provide a new preview when a fix is needed.
3. **Public-beta preparation:** complete the eleven areas below, including
   reliability, measured gaming performance, account lifecycle, final PR
   integration, token renewal, documentation and publication. This can take
   several further passes; a successful family trial does not close these gates.

## Current evidence

| Area | Evidence and limit |
| --- | --- |
| Ordinary desktop listening | The user confirmed library loading and audible catalog playback on optimized `9cb3a36`, then confirmed library refresh and audible music in the running `0745e94` preview. Its executable hash matches the clean package manifest. This does not verify later Play next/API-recovery changes, fresh-account sign-in or full-track/gaming acceptance. |
| Authorization baseline checks | `a15ba42`: Windows formatting, strict default/demo all-target Clippy, 1,017 default and 1,055 demo library tests, remaining binary/integration tests, doctests, strict Rustdoc, demo build, gettext and bridge/signer tests pass. Six opt-in tests were ignored per ordinary library suite; the muted account check passes separately. Standalone host formatting, strict Clippy, tests/build and the native credential-store round trip with dummy grants pass. |
| Motion follow-up checks | `fix/reduce-motion-controls`: two new regressions reproduced switch/tint bypasses, then passed with the shared helpers. Strict default/demo Clippy and all-target tests pass with 1,019/1,057 library tests and six opt-in tests ignored in each suite. Documentation, demo build, gettext, bridge/signer and Flatpak metadata checks pass; Linux launcher/POSIX and Windows symlink-privilege limits remain recorded. This interface-only change does not repeat fresh sign-in or live account writes. |
| Hosted checks | PR #51 at `9f4cd2a`, PR #52 at `84eeeaf` and PR #53 at `6aa2340` pass all seven jobs, including quality/docs, four platform test jobs and Nix. PR #54 at `7d25258` passes all seven jobs. PR #55's [candidate CI](https://github.com/jackkayser2005/applifast/actions/runs/38323412808) at `0745e94` passes all seven jobs too. PR #56 at `bd361d8` also passes all seven [candidate jobs](https://github.com/jackkayser2005/applifast/actions/runs/38326809797). PR #57's [API recovery CI](https://github.com/jackkayser2005/applifast/actions/runs/38482784297) at `9865001` passes all seven jobs. These results do not establish fresh-account or gaming runtime acceptance. |
| Private installer | `7d25258` installer/ZIP: clean source manifest, six entries, payload/checksum/PE/default-feature-help checks pass. Dummy-token packaging and isolated install, upgrade, shortcuts and uninstall fixtures pass against the optimized build. The local-history/package audit finds no complete private keys or MusicKit developer JWT patterns outside the declared companion token. Those fixtures never authorize or launch the app. This is a local private download, not a published GitHub release. |
| Credential/package audit | At `a15ba42`, all 55 published origin branch tips matched fetched refs. All local refs/reflogs and historical paths were inspected: 5,932 blobs, with no complete private-key blocks or MusicKit developer-JWT candidates. Integration secret patterns and original-checkout actual secret paths remain ignored/untracked. The private ZIP contains only the intended app JWT in its declared companion file. [Scope and limits](private-preview-audit.md) remain explicit; this does not prove absence of every possible secret or replace the final release audit. |
| Native host and account automation | The isolated SDK startup checks and 6.51-second idempotent Favorites check pass. The [state-conversion fix](paused-restoration.md) corrects SDK stopped/ended/stalled handling; the complete muted test now passes in 44.86 seconds, including actual saved-position resume and both retained queue occurrences. The initial failed album-detail read remains recorded. Existing-profile asset-location failure, real app restart, new-favorite writes and fresh-PC authorization remain distinct unresolved gates. |
| Full-track and mixed occurrence check | At `84eeeaf`, the separate Windows ignored test passed in 190.40 seconds. A catalog song and its catalog-linked library entry each played to completion with declared duration 90.013 seconds, maximum actual SDK position 90.00 seconds and elapsed playing time 90.15 seconds. Volume remained zero and no seek was issued. The production `State` coordinator selected occurrence 2 (manual), then occurrence 1 (context duplicate), which started and paused; all three occurrences remained distinct. The owned workspace profile restored the protected grant, with no account writes. This does not establish audible desktop listening, genuine cloud-only uploads, matched-file provenance, shuffle/repeat, fresh sign-in, application restart or gaming performance. |
| Authorization cancellation | The [new regression](authorization-cancellation.md) reproduced sign-out blocked by an unresolved SDK authorization promise. It now covers cancellation, retry, late completions, publication races and sign-out during configuration. The updated native host passes restored-account reads/search/playback/seek/pause/resume in 43.80 seconds using an owned workspace profile, subsequently removed. It sent no popup, Sign out or account-write command. Actual Apple popup behavior, expiry/revocation and profile-clearing acceptance remain open. |
| Queue removal | The [extended native check](queue-removal.md) passes in 45.43 seconds, including removing one duplicate while muted playback continues at the saved position and retaining the other queued occurrence. Default/demo tests pass with 1,021/1,062 library tests and six opt-in tests ignored per suite; strict Clippy, docs, gettext, bridge/signer and Flatpak metadata checks pass. Twenty-four inspected native captures and static HTML selectors cover main/mini-player menus in both themes and sizes. Deterministic tests cover atomic invalid batches, duplicates, repeat/shuffle, stale generations, restoration and menu clicks. Hosted CI, browser rendering, fresh-PC UI use, audible playback and gaming acceptance remain open. |
| Resource budget | The standalone probe exceeded the original 250 MiB private-memory limit: 285.406 MiB playing and 278.297 MiB paused. Initial integration was approved with that footprint. An [uncontrolled integrated baseline](integrated-performance.md) on the long-running `0745e94` preview finds 546.250 MiB peak aggregate private memory, including 354.711 MiB in the native app and 191.539 MiB in its six WebView2 processes at that peak. Average total CPU is 0.001626%, with paused UI snapshots before/after but no verified minimized or continuous playback state. Latest-preview minimized playing/paused samples and the one-hour gaming session remain unverified. Neither baseline passes the original memory gate. |
| API recovery | Fixed guidance distinguishes rejected app/account credentials, rate limits and service errors. Cached Songs rows retain a separate read error and Retry; Apple error text wraps without hiding the button. Default/demo library tests pass 1,024/1,068, with strict Clippy/docs, Node and standalone-host checks passing. Both credential-store round trips pass with dummy grants. The [read-only probe](api-recovery.md) loads 100 library songs and 10 catalog results in a fresh owned profile while the existing player is left alone. It does not explain the earlier isolated rejection or verify live credential revocation, new Play next audio behavior, fresh-account sign-in or gaming acceptance. |
| Play next | Song, selection and album menu clicks dispatch front insertion; focused tests cover duplicates, current upload identity/position, complete album pagination, out-of-order completion, failures and cancellation. Default/demo library tests pass 1,023/1,065 with six opt-in tests ignored in each; strict Clippy and docs pass. The [extended native playback check](play-next.md) currently fails before playback on the first Apple library read, after restored authorization and SDK readiness, in both feature configurations. Native credential storage passes separately with dummy grants. This is not a live Play next acceptance pass. |
| Visual evidence | Motion/Now Playing, main controls and mini-player have native comparison artifacts. The [Favorites shortcut](favorite-shortcut.md) and [Reduce motion follow-up](reduce-motion-controls.md) each have eight inspected matching captures. Switch/tint regressions reproduce the Reduce motion bypass before the fix and pass afterward; resting settings captures are pixel-identical. HTML browser rendering and real motion/runtime acceptance remain distinct from still-image and selector checks. |

## Before a public Windows beta

The original eleven areas stay in scope. Implemented behavior still needs its
real-account and release checks; passing mocks alone does not close a row.

1. **Product identity and supported controls**
   - [ ] Audit the final executable, window, mutex, installer/uninstaller,
     shortcuts/icons, storage, credential keys, browser profile and updater
     identity together. Preserve upstream credit and MIT licensing.
   - [ ] Audit every Apple screen/menu for inactive Spotify links, Connect,
     unsupported radio/playlist controls and legacy audio settings.
   - [ ] Verify explicit unsupported playback on non-Windows builds. Their
     compilation/test results do not establish Apple playback support.

2. **Sign-in and account lifecycle**
   - Native setup now shares a 45-second callback deadline and releases its
     main controller/window and COM apartment on failure. Native timeout/message
     fixtures and two real Evergreen setup/reuse checks pass without Apple grants
     or audio. See [scope and limits](native-setup-deadline.md). This does not close
     the fresh-PC, popup or real sign-out gates below.
   - [ ] Complete fresh-PC authorization, popup cancellation/retry, revoked or
     expired app/user credentials, token replacement and sign-out/re-sign-in.
   - [ ] Verify Windows Credential Manager storage, profile/account-cache
     clearing, cancellation of pending work and rejection of late old-account events.
   - [ ] Finalize app-token delivery and renewal for public downloads without
     distributing the `.p8` key or a listener's authorization. Private bundled
     tokens are implemented; long-term release delivery remains a milestone.

3. **Library and catalog**
   - [ ] Exercise pagination of songs, albums, artists, playlists and both
     searches against a representative large library.
   - [ ] Verify catalog, matched and genuinely cloud-only songs separately.
     Preserve typed library/catalog identity and Apple's playback parameters.
     Missing playback parameters must leave an actionable unavailable row,
     never silently substitute a catalog recording.
   - [ ] Verify refresh, account-scoped metadata cache, invalidation and stale
     responses. Check Home's recently added and recommendation/history shelves.

4. **Daily listening and queue**
   - [ ] Verify play/pause, seeking, volume/mute, next/previous, shuffle/repeat,
     Play next, Add to queue, dragging, reordering, removal, clear and clicking
     an upcoming row. Preserve duplicate occurrences and the [queue contract](../_reference/queue.md).
   - [ ] Verify complete tracks, mixed queues, restart restoration and failure
     behavior. An unavailable or failed song must not silently drain the queue.
   - [ ] Exercise main/mini-player window recreation, tray, taskbar and media
     controls, including closing and reopening windows during playback.
     The [artwork lifetime regression](window-artwork.md) now checks repeated
     attachment to fresh egui contexts; audible native window recreation is
     still a separate acceptance step.

5. **Favorites**
   - [ ] Verify real false-to-true writes from the player bar, mini-player, song
     row, context menu, selection and B shortcut; confirm in Apple Music.
   - [ ] Check exact confirmation, partial failure/rollback, delayed metadata,
     refresh, account changes and restart persistence.
   - [ ] Verify Favorites page/filter and the configurable left-sidebar shelf
     with Show all. Unknown flags remain unknown, and favorites remain distinct
     from ratings. Removal stays absent unless a supported route is established.

6. **Playlist writes**
   - [ ] Verify create, append selected songs/whole albums/current queue, save
     queue, duplicates, writable permissions and delayed confirmation.
   - [ ] Verify failure/retry guidance avoids duplicate playlists or accidental
     repeat writes after ambiguous network results.
   - [ ] Keep rename/delete/reorder/folders/covers/collaboration absent until
     their support is verified. Creation and appending do not prove those capabilities.

7. **Settings, motion and lyrics**
   - The [hidden preset scan fix](settings-preset-scan.md) keeps Apple Settings
     from enumerating unsupported MilkDrop folders. It preserves the visible
     controls and retained Spotify preset behavior; resource acceptance remains
     separate from removing that unnecessary work.
   - [ ] Verify theme, volume, tray/startup behavior, queue restoration,
     mini-player settings and Reduce motion after restart.
   - [ ] Run PR #13's cover hover, playing indicators, seek hover, crossfade,
     heart pop, queue arrivals, art-colored header, page fade, Now Playing and
     lyric-follow behavior. Inspect motion at normal/narrow sizes and both themes.
   - [ ] Verify available LRCLIB lyrics, missing/instrumental/error states,
     scrolling and timed-line seeking. Document that these are not Apple lyrics.
   - [ ] Check decorative Ambient Pulse in main/mini-player at play, pause,
     seek, zero volume and minimized states. Keep unsupported PCM spectrum,
     oscilloscope, EQ and MilkDrop controls absent.

8. **Reliability and gaming performance**
   - The [API cancellation regression](read-cancellation.md) reproduces an
     unresolved SDK read after abort and verifies local deadline/sign-out release,
     ignored late results and no automatic mutation retry. This deterministic
     check does not close the live network-loss or account-lifecycle gates below.
   - [ ] Test network loss/recovery, auth expiry, queue failures and output-device
     changes without stale UI reversals or lost queue entries.
   - [ ] After a two-minute warm-up, sample each state for five minutes and count
     the native app plus every browser, renderer, GPU, audio and helper process.
     Report per-process private memory and aggregate memory/total CPU.
     The uncontrolled `0745e94` [baseline](integrated-performance.md) is recorded;
     it does not close the latest-preview minimized-state checks. Investigate
     the native allocation footprint before attributing the aggregate to WebView2.
   - [ ] Evaluate the original limits honestly: at most 250 MiB private memory,
     playing below 1% average total CPU, paused below 0.2%. Reduce failed results
     or record an explicit scoped release exception, never relabel them as passes.
   - [ ] Complete a one-hour gaming listening session, checking interruptions,
     memory growth, media controls and click-to-play responsiveness.

9. **Automated and visual verification**
   - [ ] Run every applicable contribution check on the final release commit,
     including all features, Node/credential tests, packaging, docs and Nix.
   - [ ] Resolve or account for the failing native account harness and rerun
     relevant real-account acceptance. Report Windows runtime separately from
     other-platform compilation and deterministic mock coverage.
   - [ ] Finish missing interaction/motion evidence, rendered HTML reviews and
     required visual approval. Keep approved appearance separate from test gates.

10. **Documentation, privacy and security**
    - [ ] Finalize install/sign-in, account/token renewal, storage/cache behavior,
      supported capabilities, WebView2 dependency and actionable troubleshooting.
    - [ ] Recheck ignored secret paths and tracked history before publishing.
      Ship no private signing key, user token, browser profile or account cache.
    - [ ] Retain no hosted backend, app telemetry, decrypted audio persistence
      or DRM circumvention. Document Apple/Microsoft and optional lyrics traffic.

11. **Packaging and publication**
    - [ ] Review/integrate the feature PR stack into the release source while
      preserving linear history and contributor credit. Do not tag an old `main`.
    - [ ] Set the beta version in Cargo/lockfile, applicable release metadata,
      Nix vendor hash and written release notes following the previous two stable
      releases. Wait for required checks on that exact commit before tagging.
    - [ ] Verify the optimized Windows installer/portable package, publisher
      signing decision, checksums, installed upgrade/uninstall and clean-PC startup.
    - [ ] Publish the prerelease, verify every artifact, checksum, note, screenshot
      and download link, and exercise token delivery. The manual preview workflow
      needs default-branch integration/configuration and its first validated run.
    - [ ] Update website/download metadata only after artifacts exist. Update
      stable Homebrew/AUR destinations only when a stable supported release applies;
      do not advertise Apple playback on unsupported platforms.

## Explicit post-release boundaries

True PCM spectrum/oscilloscope/EQ/MilkDrop, a separate native DRM engine and
advanced playlist operations without verified platform support are not beta
features. Decorative motion must not be described as real audio analysis.
Cloud-only songs remain visible and preserve their identities even when Apple
does not provide playable parameters. These boundaries do not waive the required
runtime checks for the features that the beta does expose.
