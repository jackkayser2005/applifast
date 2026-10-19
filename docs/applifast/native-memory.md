# Native memory investigation

October 19, 2026. This follows the installed preview's
[process-tree baseline](minimal-playback-host.md). The `63d85dd` family installer
predates these changes. No saving below is a complete-player resource-budget
pass, and the maintainer's running player was left untouched.

## Window ownership correction

Closing to the tray now releases window-owned softened artwork and the lyrics
backdrop immediately. Closing or attaching a recreated window also cancels an
unfinished theme reveal, releasing its screenshot texture. Ordinary theme
animation remains enabled in live windows. Account, queue and playback state are
preserved. See [window lifetime regressions](window-artwork.md).

Both new regressions fail before the relevant correction and pass afterward.
They exercise real egui texture handles and weak texture-manager references,
including pending artwork completion channels. These tests establish ownership
release, not the number of MiB returned by a real graphics driver.

## Isolated allocation diagnostic

`examples/memory_profile.rs` reuses the demo entry point with a counted System
allocator. It requires the `demo` feature and explicit `--demo`, `--demo-data`
and `--demo-shot` arguments. Capture mode skips real credential restoration,
media integration and tray setup. It prints only elapsed time and live/peak
requested Rust allocation sizes. It never inspects allocation contents.

The counters exclude allocator overhead, memory-mapped files, native driver/GPU
allocations and every WebView2 helper. Compare them with OS private bytes and
working set, rather than treating them as total application memory.

## Initial font prototype

These results describe the initial API-compatible prototype. The subsequent
[credited upstream backport](font-memory.md) supersedes it for app integration;
its measurements and package qualification are separate.

The existing `fastframe-fonts` Windows fallback cache retains selected font
files as owned byte vectors for every later window. The selected payload on
this machine totals 60,824,796 bytes. The prototype keeps the same font
selection, family order, face indices and rendering tweaks, but lets egui
borrow read-only file mappings instead of keeping those private copies.

This is a dependency fix based on `crmne/fastframe` v0.4.1, revision
`23d87e048185ef317f272e2a67416f516b2f2b04`. It uses the existing `memmap2`
dependency. No dependency source is vendored into Applifast, and no local-path
override or lockfile change is committed. Integration requires a maintainer-owned
fork, an immutable revision pin and an upstream contribution, as required by
`AGENTS.md`.

The Windows mapping holds a read handle that permits reads and deletion/path
replacement, but denies in-place writes for the borrowed slice's lifetime.
Mapped bytes must match the scanned snapshot. An unavailable or changed file
falls back to the owned snapshot. The public low-level `fallbacks()` API still
returns owned bytes when explicitly requested. Other platforms retain their
existing loading path. A font updater that rewrites an in-use file in place may
need the app closed first. No new network destination or persistent app file is
introduced.

### Matched Windows demo measurements

Two pairs use the same original executable location, isolated shared demo data,
dark Songs view, 1100 by 760 window, and capture after twelve seconds. Each run
exits successfully. Values are medians between 5 and 11.5 seconds; MiB means
1,048,576 bytes. The prototype and baseline differ in fallback font storage.
These short debug-demo samples include no Apple account, playback host or audio.

| Pair / font storage | Private MiB | Resident MiB | Counted Rust heap MiB |
| --- | ---: | ---: | ---: |
| 1 / owned | 173.13 | 175.10 | 69.96 |
| 1 / mapped | 114.57 | 174.79 | 11.96 |
| 2 / owned | 175.91 | 174.09 | 69.96 |
| 2 / mapped | 116.46 | 174.47 | 11.96 |

This reduces private memory by 58.57 and 59.46 MiB respectively, while resident
working set stays around 175 MiB. The mapped font pages remain resident when
used. Discovery still initially reads fonts, so this does not establish a lower
startup peak. Both matched screenshot pairs have zero differing pixels; that
single theme/size is not multilingual, narrow-window or interactive acceptance.

The baseline profiling executable has SHA256
`96e6da1fbe8d9fb34cb40f4794045e7f311c93021e0dc0f9df376b149a5c0094`.
The mapped executable has SHA256
`6d27ebad998a6be8f25cabbb2e1936eeb13ee198df09499182a50b287c33566f`.
Local raw-report hashes, in table order:

| Sample | SHA256 |
| --- | --- |
| 1 / owned | `42e76eebfdd0a79dcf132a99d9fc114e5c6f17456708aa6b4bf9f8d52ad02475` |
| 1 / mapped | `b06459dab7553d83a24896ac39ce8385c24c69bd0e7550d15983b5d4b2d12f78` |
| 2 / owned | `aec3bfff5c936548e0f2eb437b7b781e26d074ac632ed13e39bc5399471e255a` |
| 2 / mapped | `154ff8f082db44efa0ce66be4e241d7167ea0f3457ee594752ba4027725d9f62` |

An earlier run from a renamed copy timed out and was excluded. Its cause is
unresolved; every reported pair uses the original executable location.

## Checks and remaining gates

The app changes pass local Windows default/demo all-target tests (1,031/1,077
library tests, seven opt-in checks ignored per suite), strict Clippy, doctests,
strict demo Rustdoc, demo build, formatting, gettext and the Node bridge/token
self-checks. The profiling example's allocator and isolation guards have focused
tests. Local all-feature MilkDrop, other-platform tests, website and Nix checks
remain hosted gates on the submitted revision.

The font prototype separately passes 39 font-library tests, with three ignored,
and strict all-target/all-feature font Clippy. Its full workspace checks and
cross-platform CI are separate from Applifast's checks and must be recorded
before integration. Tests cover identical font definitions, removal of the
owned payload, write exclusion and fallback to a scanned snapshot.

After integration, rebuild and qualify an optimized installer. Repeat native
main/mini/tray recreation and equivalent real-library navigation, then measure
the complete process tree after a two-minute warm-up for five minutes in each
minimized playing and paused state. Keep the 250 MiB private-memory, under 1%
playing CPU and under 0.2% paused CPU limits, plus a one-hour gaming trial.
Fresh-PC authorization, full tracks and device/network recovery remain required.
