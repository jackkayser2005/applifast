# Large listening contexts

Selecting a song failed when the loaded context plus manual additions exceeded
1,000 occurrences. The selected row could be valid and playable, but the native
model rejected the entire context before contacting MusicKit.

The local queue now accepts up to 100,000 occurrences, matching the existing
library metadata count bound. The native model remains the owner of manual
additions, duplicate occurrences, context, Previous history, shuffle, repeat and
automatic advancement. Above the original 1,000-item bulk boundary, a validated
`nativeQueue` command carries only the selected item, occurrence index, count and
transport state. Metadata-only edits keep the current track loaded. State
messages omit the redundant full order, preserving Rust's authoritative order.
The host does not independently choose a next song in this mode.

Smaller queues retain their existing protocol. Transitioning between modes
preserves the current item's identity. Original library IDs and playback
parameters remain separate from catalog matches. A rejected command does not
drain the queue. The command and event byte limits, legacy diagnostic bulk queue
limit, playlist-write batch limits and authorization-generation checks remain.
Occurrence-order validation now reuses membership sets instead of repeatedly
scanning the context for every upcoming row.

Initial compact playback errors carry the same playback-generation tag as
ordinary play errors. An older failed selection can therefore be rejected
without canceling a newer selection or an independent library refresh. The
error remains actionable and the local queue is retained.

## Storage and network

The session JSON schema remains version 1. New builds still read existing small
sessions and accept larger valid queues. Older previews reject a saved queue
above their own 1,000-item limit. The existing atomic save and 32 MiB file bound
remain; sufficiently large metadata snapshots can exceed that bound and report
a save error. This is not unlimited caching. No audio or credentials enter the
session file, and no new endpoint, backend, dependency or telemetry is added.

This change does not fetch unloaded collection pages, guarantee unavailable
uploads, establish a resource-budget pass or alter the interface layout.

## Verification

- The new native model regression fails before the fix when selecting beyond
  the first 1,000 entries. It passes afterward with 1,005 context entries and two
  manual duplicates.
- It checks typed playback parameters, automatic advancement to a manual
  occurrence, stale-end rejection, exact removal, serialized restoration of the
  full order and a paused position.
- Protocol tests check a 100,000-item order and compact-command bounds. Windows
  boundary coverage checks command parsing and sanitized large-index state.
- Bridge fixtures exercise one-item SDK loads, metadata edits without reload,
  native ownership of advancement, paused seek, invalid-command retention and
  transition back to the original bulk protocol. These fixtures do not establish
  real-account playback or resource measurements.
- A forced SDK rejection reproduces a missing generation on the initial compact
  play error before the follow-up fix. Direct play now retains generation 47,
  and an intent-wrapped request retains generation 48. The failure message does
  not expose the SDK response. The native model's existing error regressions
  protect stale selection and in-progress metadata refresh state.
- On October 18, 2026, the focused real-account Windows regression passes in
  10.06 seconds using an isolated app-owned profile and restored local grants:
  1,001-item playback, metadata edits without restart, advancing to a manual
  duplicate and a paused two-second restore. Playback is muted; no sign-in
  popup, sign-out or account mutation is performed. This is a short transport
  check, not full-track listening, audible-output or performance acceptance.
- The normal test profile fails local page navigation before Apple initialization
  (`success=false`, WebView error status 0, HTTP status 0). The isolated profile
  loads the same embedded assets with HTTP 200. The normal profile is preserved.
  Runtime version 155.0.4283.62 is installed alongside earlier patch versions;
  the exact selected version was not recorded by this playback run.
- In the isolated profile, the broader native regression reaches generation 13
  and fails its earlier small-queue Next check after seek/restore: stopped state,
  zero duration and queue length three. It has not yet sent the new compact
  command at that point. A later repeat on unchanged `72eb9c0` passes the entire
  sequence, including its compact-context checks, in 53.44 seconds. The first
  playable library row supplies catalog playback metadata and a 169,409 ms
  duration. The earlier failure remains unexplained and intermittent; a passing
  repeat does not establish sustained listening reliability.
- Local Windows checks pass both at `72eb9c0` and after the error-generation
  follow-up: formatting, strict default/demo all-target Clippy,
  default/demo all-target tests, doctests, strict demo Rustdoc, demo build,
  gettext consistency and Node bridge/signer checks. The library suites pass
  1,029/1,075 tests with seven opt-in checks ignored in each ordinary suite.
  Standalone host checks pass strict Clippy and 24 tests, with its dummy native
  credential-store check ignored in the ordinary run. Credential storage is
  unchanged by this slice.
- The optimized private `72eb9c0` installer and ZIP pass clean-source, payload,
  checksum and executable checks, dummy-token packaging, and isolated per-user
  install/upgrade/shortcuts/uninstall fixtures. The raw Git/package pattern audit
  has no secret-pattern hits outside the declared companion app token, subject
  to its opaque/encoded/remote-history limits. The unsigned preview is version
  0.12.0 and its token expires November 6, 2026. That package precedes the
  playback-error generation follow-up.
- Hosted all-feature/platform/site/Nix checks on the final source remain
  required. The local runs
  above do not establish MilkDrop feature coverage, Linux/macOS runtime support,
  full listening acceptance or a resource-budget pass.
