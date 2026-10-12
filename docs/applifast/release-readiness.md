# Applifast Windows beta readiness

Snapshot: October 12, 2026. This is the release checklist, not a release claim.
The candidate code is `4f47613` (PR #48), adding an SDK startup deadline on top
of PR #46's Favorites shortcut and PR #47's checklist. `main` remains at
`995c768`; these preview changes have not
been merged into a public release.

## Next private trial: Dad's PC

The next handoff is an invited Windows x64 preview. A tester needs an Apple
Music subscription, network access and WebView2 Evergreen. The package supplies
the app developer token; the tester authorizes their own Apple account. They
do not need Developer Program membership, a signing key, Rust, Node, or commands.
Use the [Windows tester guide](testing.md).

- [x] Build and verify the optimized `4f47613` installer/ZIP, checksums,
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
  authorization. The complete test still fails at paused queue restoration.
- [ ] Resolve the native paused-restoration timeout and verify resume starts at
  the saved position without losing queue occurrences.
- [ ] Verify every required hosted CI job on the candidate commit.
- [ ] On Dad's PC, install through the wizard, open from Start, authorize in
  Apple's popup and load his library without copying another user's profile,
  user token or account cache. Check cancel and retry before successful sign-in.
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

1. **Private handoff:** finish candidate CI, then try the installer and the
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
| Ordinary desktop listening | The user confirmed library loading and audible catalog playback on optimized `9cb3a36`. This does not verify the later Favorites changes or full-track/gaming acceptance. |
| Latest local checks | `4f47613`: Windows formatting, strict default/demo all-target Clippy, 1,016 default and 1,054 demo library tests, remaining binary/integration tests, doctests, strict Rustdoc, demo build, gettext and bridge/signer tests pass. Five opt-in tests were ignored per ordinary library suite. The earlier separate native credential-store check passes using dummy grants; this change does not alter credential code. |
| Hosted checks | PR #46 at `a056b19`: all seven [CI jobs](https://github.com/jackkayser2005/applifast/actions/runs/38269332681) and every applicable [Packaging job](https://github.com/jackkayser2005/applifast/actions/runs/38269334505) pass. PR #48's [candidate CI](https://github.com/jackkayser2005/applifast/actions/runs/38276771885) at `1410e27` has six successful jobs; Windows x64 remains in progress at this snapshot. |
| Private installer | `4f47613` installer/ZIP: clean source manifest, six entries, payload/checksum/PE/default-feature-help checks pass. Dummy-token packaging and isolated install, upgrade, shortcuts and uninstall fixtures pass. Those fixtures never authorize or launch the app. This is a local private download, not a published GitHub release. |
| Credential/package audit | At `1410e27`, all 50 published origin branch tips matched fetched refs. All local refs/reflogs and historical paths were inspected: 5,904 blobs, with no complete private-key blocks or MusicKit developer-JWT candidates. Original/integration ignore checks pass. The private ZIP contains only the intended app JWT in its declared companion file. [Scope and limits](private-preview-audit.md) remain explicit; this does not prove absence of every possible secret or replace the final release audit. |
| Native host and account automation | The isolated [SDK startup checks](listening-slice.md#retry-after-startup-failure) pass for normal loading and a blocked SDK. The corrected idempotent Favorites check passed in 6.51 seconds with protected authorization restored into a workspace profile. A separate muted test passed reads, search, playback, seek and pause, then failed at paused restoration: SDK loading, actual position/duration zero, two retained entries. The existing-profile asset-location failure is narrowed but not fully explained. New-favorite writes, successful restoration and fresh-PC authorization remain unchecked; see [account gate evidence](favorites-write-evidence.md). |
| Resource budget | The standalone probe exceeded the original 250 MiB private-memory limit: 285.406 MiB playing and 278.297 MiB paused. Initial integration was approved with that footprint. Integrated five-minute samples and the one-hour gaming session remain unverified. Roughly 300 MB reported by the user is not a measurement against the original gate. |
| Visual evidence | Motion/Now Playing, main controls and mini-player have native comparison artifacts. The [Favorites shortcut](favorite-shortcut.md) has eight inspected matching captures. HTML browser rendering and real motion/runtime acceptance remain distinct from still-image and selector checks. |

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
   - [ ] Test network loss/recovery, auth expiry, queue failures and output-device
     changes without stale UI reversals or lost queue entries.
   - [ ] After a two-minute warm-up, sample each state for five minutes and count
     the native app plus every browser, renderer, GPU, audio and helper process.
     Report per-process private memory and aggregate memory/total CPU.
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
