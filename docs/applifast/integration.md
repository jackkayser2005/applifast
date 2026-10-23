# Windows Apple Music source integration

This integration branch assembles the Windows port and its focused follow-ups
for review against `main`. It is not a published release or a runtime acceptance
claim. The authoritative remaining gates are in [release readiness](release-readiness.md).

## Source and scope

The refreshed `origin/main` baseline is `995c768dcba4da7f93302bf2107af6b7ec3df02b`.
The assembled application source is `8da79db02b980f7f12f57c99382ede50eaef87b9`,
with 136 linear commits after that baseline and no merge commits. Individual
feature PRs stay available for focused review. This branch targets `main` without
moving another feature branch's base or rewriting published history.

The scope includes the Apple listening work from PR #6, design/polish from
PRs #1 through #12, adapted motion from PR #13 and the subsequent Apple feature,
reliability, evidence and packaging slices through PR #89. The current native
listening architecture remains Rust/egui plus an independent MusicKit JS/WebView2
STA host. Spotify-specific implementation remains retained upstream code;
Apple mode exposes the supported Apple controls. Linux/macOS playback stays
explicitly unsupported, with compilation tested separately.

The application changes cover authorization/token onboarding, typed catalog and
library reads, Home/search/favorites, supported playlist creation/appends, local
queue/session restoration, main/mini-player/tray/media controls, LRCLIB lyrics,
supported settings, local playlist pins and private Windows installer/ZIP builds.
See [the detailed capability boundary](listening-slice.md). This assembly adds no
new capability or redesign beyond those reviewed feature slices.
The assembly audit additionally repairs one PR #13 interaction: optimistic Apple
favorites switched to a read-only filled image that bypassed the existing pop
scale. Player and row filled images now consume that same scale, within their
existing cells. Their read-only behavior and write handling are preserved.

The October 22 refresh fast-forwards the previous integration head `64db4c3`
without rewriting history. It brings in the minimal hidden host, maintainer-owned
font-memory backport, size-specific artwork and cache limits, profile clearing
and setup/retry fixes, large listening contexts, protected fork publishing,
approved responsive layouts (#82/#83), native indicator frames (#84), favorite
shortcut feedback (#85), and the guarded account playlist check (#86).
See the [prioritized roadmap](roadmap.md) and linked evidence for each scope.
These source changes do not by themselves establish the whole-player budget.

The follow-ups add the unchanged strict quality job's cold-build timeout fix,
mini-player heart feedback (#88), and consistent library continuation page sizes
(#89). All seven hosted jobs pass on application source `8da79db` in
[run 39838320991](https://github.com/jackkayser2005/applifast/actions/runs/39838320991).
The separate refreshed integration-context run must still finish on that head.
The [current preview recheck](apple-user-guides.md) records native transport,
full-track and paused resource results. CPU passes its observed paused target;
the 250 MiB aggregate private-memory gate remains failed.

MIT licensing and upstream credit remain. No signing key, listener token,
browser profile or account cache belongs in a distributable. The package's
declared app developer JWT is intentional; [audit limits](private-preview-audit.md)
remain applicable. There is no hosted backend, app telemetry, DRM bypass or
decrypted-audio persistence.

## PR #13 motion trace

The motion implementation was adapted into Apple playback in `db12b68`, then
extended with Apple lyrics in `2c6c576` and shared Reduce motion fixes in PR #54.
The original PR head `6060d0e` is not an ancestor of this branch. Its implementation
is traced through those adapted commits and connected call sites below; an open
original PR is not evidence that the behavior is absent from the preview.

| Requested item | Current implementation |
| --- | --- |
| Reduce motion in Settings | `Settings::reduce_motion`, the settings control, and `ui::motion::set_reduced` applied by the shared interface. [Follow-up evidence](reduce-motion-controls.md) covers switches, tint and drag feedback. |
| Play on cover hover with lift | Cover/card drawing and hover actions in `src/ui/widgets.rs`, using the shared motion policy. |
| Dancing playing indicators | `ui::motion::playing_bars` and its song/card callers. These are decorative fixed patterns. |
| Seek bar and hover knob | `ui::widgets::thin_slider`, reused by transport and Now Playing. |
| Cover crossfade | `ui::player_bar::crossfaded_cover`, retaining the previous ready image while a replacement loads. |
| Heart pop | Shared `pop_heart`/`heart_scale` and Favorites control callers. Real favorite writes still need their account acceptance. |
| Queue rows arriving | `ui::motion::note_rows`/`arrival` with occurrence keys in `src/ui/queue.rs`; duplicate rows have independent keys. |
| Art-colored readable header | Collection/artist header drawing and palette/color helpers, with clamped artwork tint. |
| Full-screen Now Playing controls | `src/ui/lyrics.rs` and the shared native transport actions. |
| Smoother timed lyrics | Shared motion-aware scroll/line transitions in `src/ui/lyrics.rs`; Apple mode uses LRCLIB when available. |
| Page fade | `ui::motion::page_opacity` applied in `src/ui/mod.rs`. |

Deterministic motion, slider, queue and lyrics tests exercise these paths. Stills
can inspect appearance, hover endpoints and Reduce motion resting states. They
cannot prove live transition timing, lyric-follow behavior, audible controls or
gaming performance. Those interactive checks stay open. Ambient Pulse is also
decorative; Apple mode does not expose the retained PCM spectrum/EQ/MilkDrop controls.

The [native frame follow-up](native-motion-frames.md) now captures the real playing
indicator over time: eight distinct states with motion enabled, and identical
indicator pixels in eight Reduce motion frames, across both themes and sizes.
This qualifies that specific native render path, without claiming the other
transitions, lyrics, controls, account behavior or resource budget are accepted.

## Review and integration gates

The maintainer requested importing the design PRs and PR #13 and approved the
existing preview style in the conversation. That scope authorization is retained;
it does not establish every visual, account or release test as passing. The
[comparison index](reviews.html) identifies individual evidence and limitations.
The clean baseline build and overall native captures are complete. The earlier
motion-specific gallery remains candidate-only; the overall comparison below
does not replace its missing live interaction evidence.

The [overall native comparison](review-integration/index.html) now provides 32
captures from clean main and the current integration renderer: Songs, queue,
Settings and mini-player in both themes and main-window size requests. Main
captures match 1100×760/760×760. The built-in mini fixture fixes 2× scale, producing
550×232 at both requests; it does not prove multiple mini scales. Metadata is the
same shared sample fixture, with documented service identity, favorite and
supported-column differences. All captures were inspected, with full narrow
Settings and queue views inspected separately. Those original narrow captures
showed a crowded central pane and an overlapping toolbar. The
[responsive toolbar follow-up](narrow-toolbar.md) addresses the Search/control
overlap with a second row in narrow Apple panes. A
[collection follow-up](narrow-collections.md) addresses the cramped header,
filter/actions and row names. Its own comparison separates the new responsive
scope from the preceding toolbar change. The maintainer approved both #82 and #83
on October 22, 2026: "Approve both layouts". Their geometry/click regressions and
native/browser comparisons pass; real interactive use remains a separate gate.

The filled-heart regression uses actual row/player clicks and painted image
geometry. It reproduces the fixed 16-pixel filled row while the pop should be
active, then passes with the existing scale applied. Both palette variants and
760/1100 widths cover interruption by Reduce motion, normal size after re-enabling
motion, and filled hearts retaining no click action. It uses sample data with no
live account writes. At that earlier assembly, local app checks passed 1,026 default and 1,072
demo library tests, with six opt-in checks ignored per suite, plus the existing
strict Clippy, formatting, all-target/binary/integration, doc, gettext and Node
checks. Translation reference updates preserve all message semantics.

The assembly application at `833edfa` passes Windows formatting, strict
default/demo Clippy, all-target default/demo tests, doctests, strict demo Rustdoc,
demo build, gettext and Node bridge/signer checks. The library suites pass 1,026
default and 1,071 demo tests, with six opt-in checks ignored per ordinary suite.
Standalone host checks pass 21 tests and strict Clippy; two real empty-profile
Evergreen starts and immediate reuse pass without Apple grants or audio. The
optimized installer/ZIP pass the checks recorded in PR #62. These results apply
to that exact source and do not establish the eventual integration/release commit.

The refreshed application at `6357c67` passes Windows formatting, strict
default/demo all-target Clippy, default/demo all-target tests (1,041/1,089 library
passes, nine opt-in checks ignored per ordinary suite), doctests, strict demo
Rustdoc, demo build, gettext and Node bridge/token checks. The separately
authorized [native playlist check](local-account-checks.md) creates and appends
two exact library identities successfully. Saved-account reads and idempotent
existing-favorite checks pass too. New favorites, fresh sign-in, normal desktop
write interaction and recovery remain separate gates. Required hosted jobs must
pass on the refreshed integration head, rather than relying on parent PR results.

- [x] Complete matching overall light/dark and normal/narrow main captures,
  with fixed-size mini captures and their limits stated.
- [x] Complete narrow toolbar/collection responsiveness, matching native and
  browser comparisons, and explicit visual approval for #82/#83.
- [ ] Finish remaining live interaction/motion evidence and any outstanding
  visual scope. Static selector checks and scripted native indicator frames
  cover their stated subsets only.
- [ ] Pass every required hosted check on the integration head, including the
  full feature set, platform jobs, documentation site and Nix.
- [ ] Integrate through the maintainer-approved linear workflow and verify the
  resulting `main` source. Keep feature attribution and source provenance.
- [ ] Configure and validate the manual Windows preview workflow after its
  default-branch integration, using an app JWT without a signing key or user grant.
- [ ] Finish the original eleven [release areas](release-readiness.md), including
  Dad's fresh-PC setup, live writes/listening/account lifecycle, resource limits,
  gaming soak, beta version/metadata, signing decision and verified publication.

The 250 MiB private-memory gate remains failed in existing measurements. Initial
private listening approval at roughly 300 MiB is not a public release exception.
No release tag, updater activation or public download claim follows from opening
this integration PR.
