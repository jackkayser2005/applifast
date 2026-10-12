# Applifast Windows beta readiness

Snapshot: October 12, 2026. This is the release checklist, not a release claim.
The candidate code is `292ab76` (PR #51), fixing paused restoration and MusicKit
state conversion on top of the SDK startup deadline and Favorites shortcut.
`main` remains at
`995c768`; these preview changes have not
been merged into a public release.

## Next private trial: Dad's PC

The next handoff is an invited Windows x64 preview. A tester needs an Apple
Music subscription, network access and WebView2 Evergreen. The package supplies
the app developer token; the tester authorizes their own Apple account. They
do not need Developer Program membership, a signing key, Rust, Node, or commands.
Use the [Windows tester guide](testing.md).

- [x] Build and verify the optimized `292ab76` installer/ZIP, checksums,
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
| Latest local checks | `292ab76`: Windows formatting, strict default/demo all-target Clippy, 1,017 default and 1,055 demo library tests, remaining binary/integration tests, doctests, strict Rustdoc, demo build, gettext and bridge/signer tests pass. Five opt-in tests were ignored per ordinary library suite; the complete muted account check passes separately. The earlier native credential-store check passes using dummy grants; this change does not alter credential code. |
| Hosted checks | PR #46's seven CI jobs and applicable Packaging jobs pass; PR #48 at `1410e27` also passes all seven CI jobs. PR #51's [latest CI](https://github.com/jackkayser2005/applifast/actions/runs/38285671862) at docs-only follow-up `9f4cd2a` has successful quality/docs jobs; Nix and the four platform test jobs remain in progress at this snapshot. |
| Private installer | `292ab76` installer/ZIP: clean source manifest, six entries, payload/checksum/PE/default-feature-help checks pass. Dummy-token packaging and isolated install, upgrade, shortcuts and uninstall fixtures pass against the optimized build. Those fixtures never authorize or launch the app. This is a local private download, not a published GitHub release. |
| Credential/package audit | At `292ab76`, all 53 published origin branch tips matched fetched refs. All local refs/reflogs and historical paths were inspected: 5,918 blobs, with no complete private-key blocks or MusicKit developer-JWT candidates. Integration secret patterns and original-checkout actual secret paths remain ignored/untracked. The private ZIP contains only the intended app JWT in its declared companion file. [Scope and limits](private-preview-audit.md) remain explicit; this does not prove absence of every possible secret or replace the final release audit. |
| Native host and account automation | The isolated SDK startup checks and 6.51-second idempotent Favorites check pass. The [state-conversion fix](paused-restoration.md) corrects SDK stopped/ended/stalled handling; the complete muted test now passes in 44.86 seconds, including actual saved-position resume and both retained queue occurrences. The initial failed album-detail read remains recorded. Existing-profile asset-location failure, real app restart, new-favorite writes and fresh-PC authorization remain distinct unresolved gates. |
| Full-track and mixed occurrence check | The separate Windows ignored test passed in 190.40 seconds. A catalog song and its catalog-linked library entry each played to completion with declared duration 90.013 seconds, maximum actual SDK position 90.00 seconds and elapsed playing time 90.15 seconds. Volume remained zero and no seek was issued. The production `State` coordinator selected occurrence 2 (manual), then occurrence 1 (context duplicate), which started and paused; all three occurrences remained distinct. The owned workspace profile restored the protected grant, with no account writes. This does not establish audible desktop listening, genuine cloud-only uploads, matched-file provenance, shuffle/repeat, fresh sign-in, application restart or gaming performance. |
| Full-track regression checks | The tests/docs-only branch passes Windows formatting, strict default/demo all-target Clippy, 1,017 default and 1,055 demo library tests, remaining targets, doctests, strict Rustdoc, demo build, gettext and Node bridge/signer checks. Six opt-in tests are ignored per ordinary library run; the full-track native result is separate. The disposable browser profile was removed after shutdown, retaining the protected grant. No production behavior or package changed; the verified private download remains `292ab76`. Hosted all-feature and other-platform checks for this test change remain pending. |
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
