# File-backed fallback fonts

The font-memory follow-up pins every fastframe crate, including the build-time
catalog compiler, to the maintainer-owned fork at
`ba3db2d36f7b08f5d99e650c4adcaa05f90cda87`. This retains the v0.4.1 foundation
and backports Darkhan Akhmetov's upstream
[font mapping correction](https://github.com/crmne/fastframe/commit/4cf12984fca3ade655b214add154828684e3e993),
with its original author and cherry-pick reference. It does not upgrade to
fastframe 0.13 or bring in unrelated foundation changes.

Selected fallback font files stay in read-only mappings instead of complete
private heap copies. Font families, ordering, face indices and metrics keep
their existing selection rules. The two retained MilkDrop/Winamp font consumers
now accept borrowed font bytes directly. Apple mode continues to hide
unsupported PCM controls. MIT and bundled-font licenses remain unchanged; no
dependency source is vendored into Applifast.

Windows mappings retain read handles for their lifetime, allowing reads and
path replacement while denying in-place writers. A regression checks existing
and future writers, successful path replacement, stable original bytes and
write access after the mapping is dropped. An updater that rewrites an in-use
font in place may need the app closed first. Other platforms retain upstream's
mapping behavior and need their own compilation/runtime coverage.

The dependency backport passes all required local Windows workspace checks:
formatting, strict default/all-feature and minimal-feature Clippy, default and
all-feature all-target tests, doctests, minimal log/font tests and strict
all-feature Rustdoc. The font suite passes 38 tests with three opt-in checks
ignored. Cross-platform CI remains required.

The lifetime protection is also ported onto current upstream `main` at
`178b3ebcdcb683a3c3a65b7a067e61d746d33864`, with every required Windows workspace
check passing there. The [upstream comparison](https://github.com/crmne/fastframe/compare/main...jackkayser2005:fix/windows-font-mapping-locks)
is one commit ahead with no unrelated foundation changes. Automated PR creation
received an integration-permissions rejection (HTTP 403); submission remains
pending through GitHub's form. The core memory-mapping correction was already
contributed and merged upstream as #17.

## Matched native measurements

The credited backport has its own eight successful twelve-second Windows demo
runs, separate from the [initial prototype](native-memory.md). Each before/after
pair uses the original profiler location, identical cached sample artwork and
Songs data, an isolated data directory, a neutral pointer and the same theme
and window size. No Apple host, account or audio is started. The following are
medians between 5 and 11.5 seconds, in MiB (1,048,576 bytes).

| Theme / size | Private before | Private after | Resident before | Resident after |
| --- | ---: | ---: | ---: | ---: |
| Dark / 1100 by 760 | 172.91 | 115.64 | 174.52 | 117.11 |
| Dark / 760 by 760 | 175.20 | 117.30 | 179.55 | 118.64 |
| Light / 1100 by 760 | 174.54 | 116.45 | 174.29 | 116.95 |
| Light / 760 by 760 | 175.34 | 115.68 | 179.91 | 118.39 |

Private memory falls by 57.27 to 59.66 MiB, and resident memory by 57.34 to
61.52 MiB. A separate single private-working-set counter in each run, sampled
about 5.5 to 7 seconds after launch, falls from 122 to 125 MiB to 65 to 67 MiB.
Counted live Rust heap falls from roughly 70 to 72 MiB to 12 to 13 MiB. Used
mapped pages can still be resident; private bytes, working set and GPU memory
remain distinct measures. These short demo results do not establish the
complete-player resource budget or optimized real-account performance.

All four [native screenshot pairs](review-font-memory/index.html) have zero
differing pixels; each candidate capture was visually inspected. The sample
does not exercise every script, mini-player font or interactive motion. Static
HTML selector checks remain separate from actual browser rendering.

The frozen baseline executable has SHA256
`96e6da1fbe8d9fb34cb40f4794045e7f311c93021e0dc0f9df376b149a5c0094`;
the pinned candidate has SHA256
`f25070e9feab740ee7eb60b5a9e669f4bf20fbb9840433b5f14ed3b0d89f2ec1`.
The [machine-readable results](font-memory-samples.json) record source inputs,
dependency pin, raw-report hashes, all counters and pixel comparisons.

Local Windows app formatting, strict default/demo all-target Clippy, all-target
tests (1,031/1,077 library tests, eight opt-in checks ignored per suite),
doctests, strict demo Rustdoc, demo/profiler builds, gettext and Node bridge/token
self-checks pass. The updated Nix vendor hash, hosted all-feature/platform/site
checks and optimized package qualification remain required on the final source.

The read-only embedded-host check also passes on application source `1eedaad`:
100 typed library songs and ten catalog results in 3.929 seconds, navigation
HTTP 200, with the minimal page's hash confirmed. It restores protected local
authorization in a fresh owned profile and sends no audio, popup, sign-out or
account-write commands. The Windows credential-store round trip separately
passes using only its temporary dummy entry. Neither check proves fresh sign-in
or audible playback on this candidate.

The initial [Nix discovery job](https://github.com/jackkayser2005/applifast/actions/runs/39330041761)
reports `sha256-DUDZPNu7M1yuGj0sCRRrb9F8EsFm4JiixbTyfMmmHsk=` for the updated
vendor staging output. That measured value replaces the intentional fake hash;
the full Nix build must pass on the resulting commit before merge.

At runtime, no new network endpoint, credential access, settings format, application file,
telemetry or audio cache is introduced. The family's `63d85dd` installer predates
this change. The complete-player 250 MiB and CPU/gaming acceptance gates remain
open in the [release checklist](release-readiness.md).
