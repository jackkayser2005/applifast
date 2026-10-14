# Integrated Windows performance baseline

October 13, 2026: a read-only sample of the maintainer's already running
Applifast preview found **546.250 MiB peak aggregate private memory**. This
exceeds the original 250 MiB limit. It is an uncontrolled normal-use baseline,
not a minimized playback or paused-state acceptance test.

## Source and method

The running optimized executable matches the clean `0745e94` package manifest:

- Source: `0745e94038e86170d04b2d1eea681a88a5ee0fe9` (PR #55).
- Executable SHA256: `81a4fb58194f97e03d119d602463952eaca37ae69f9d2778c618f2a5e55ba5f0`.
- Windows: build 26200; 16 logical processors.
- Associated Evergreen WebView2 file version: `154.0.4258.62`.
- App process: PID 14964, created at 14:38:32 UTC. It had been running for
  approximately nine hours when measurement began.

The existing [process sampler](../../prototypes/apple-playback/measure.ps1)
used a 120-second warm-up followed by 300.274 seconds of measurement, finishing
at 23:56:09 UTC. All 289 snapshots included the same seven process identities:
the native app and six WebView2 processes. Per-process private bytes sum to
each aggregate, the PID lists match the detailed records, and the sampler
reports no accounting failures. CPU is normalized across all 16 processors.

The app's documented `now-playing --raw` command reported paused before and
after sampling. Only that state label was retained, without song metadata.
The sampler did not observe the playback state continuously, confirm the SDK's
physical state or verify that windows were minimized. No command changed music,
volume, authorization or window visibility. Builds and heavy QA were deferred
during measurement. Normal use of this PC was not controlled.

The local raw report remains ignored at
`.cache/integrated-baseline-counters.json`, with SHA256
`c1dfa7408717b3416822e59405e96581518bd8ab29d86e0d236fda4bdfa51322`.
It contains process identities and counters, without command lines, paths,
credentials or library metadata. This is local measurement, not app telemetry.

## Results

| Counter | Observed value |
| --- | ---: |
| Minimum aggregate private memory | 545.703 MiB |
| Maximum aggregate private memory | 546.250 MiB |
| First / last aggregate sample | 545.820 / 545.961 MiB |
| Average total CPU | 0.001626% |
| Native app at aggregate memory peak | 354.711 MiB |
| All WebView2 processes at that same peak | 191.539 MiB |

| Process | PID | Mean private memory | Peak private memory |
| --- | ---: | ---: | ---: |
| Applifast | 14964 | 354.699 MiB | 354.711 MiB |
| WebView2 | 17896 | 77.753 MiB | 77.801 MiB |
| WebView2 | 18696 | 50.805 MiB | 50.805 MiB |
| WebView2 | 33692 | 40.487 MiB | 40.609 MiB |
| WebView2 | 29372 | 11.779 MiB | 11.883 MiB |
| WebView2 | 40020 | 7.660 MiB | 7.660 MiB |
| WebView2 | 12048 | 2.837 MiB | 2.840 MiB |

Individual process peaks need not occur in the same snapshot. All seven rows
were present in every snapshot. Polling still cannot see a helper created and
exited entirely between snapshots; counter completeness is subject to that
existing sampler limitation.

## What this establishes

The measured CPU value is low, but it does not close the minimized paused CPU
gate. Memory fails the original limit in every snapshot, and the native app
accounts for more private memory than all six browser processes combined.
The five-minute range is narrow; it does not establish the absence of growth
over hours or prove a leak. There is no comparable clean-start sample, so app
uptime cannot be assigned as the cause of the footprint.

The next memory investigation should compare a fresh latest preview with
repeatable library browsing and main/mini-player recreation, then identify
retained native allocations before changing cache limits. Existing artwork
and table caches already have caps. This process-level evidence does not
identify which allocation owns the native private bytes, and no cache or
playback behavior was changed based on a guess.

The newer `9865001` preview has passed all seven hosted CI jobs and has a
verified installer/ZIP, but it was not the running executable in this sample.
Its integrated minimized playing/paused samples and the one-hour gaming trial
remain required. The earlier [standalone probe measurements](playback-evidence.md)
also retain their failed memory results; neither measurement is relabeled as
a passing release gate.
