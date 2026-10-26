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

## October 15: latest installed integration preview

A second read-only normal-use sample identifies the clean `64db4c3` integration
preview, whose installed executable matches SHA256
`1d367ca97aec1396646dc87d905a249c4ba214b39f03d8a2a76d828e5dd01b4a`.
It uses Windows build 26200, 16 logical processors and Evergreen
`155.0.4283.45`. The native PID 28232 started at 21:19:01.888 UTC. A 120-second
warm-up preceded 300.769 seconds of measurement, finishing at 21:33:28.260 UTC.
All 290 snapshots include the same nine process identities: the native app and
eight WebView2 processes. Detailed private-byte sums match each aggregate and
the sampler reports no accounting failures.

The maintainer continued using the app. Playback state and minimization were
not observed or controlled. The sampler's required `StateTag` was supplied as
`paused`, but `PlaybackStateVerified` is false. That input is not evidence of
paused playback. This is not a minimized paused/playing or gaming acceptance
test. No playback, volume, queue, authorization or window command was sent.
Source checks and compilation ran separately on the PC during sampling.

| Measurement | Result |
| --- | ---: |
| Minimum aggregate private memory | 502.176 MiB |
| Maximum aggregate private memory | 503.637 MiB |
| First / last aggregate sample | 502.789 / 503.246 MiB |
| Average total CPU | 0.005844% |
| Native app at aggregate memory peak | 210.965 MiB |
| All WebView2 processes at that same peak | 292.672 MiB |

| Process role | PID | Mean private memory | Peak private memory |
| --- | ---: | ---: | ---: |
| Native app | 28232 | 210.973 MiB | 210.996 MiB |
| WebView2 browser | 34336 | 41.177 MiB | 41.633 MiB |
| WebView2 crash handler | 34824 | 2.940 MiB | 3.012 MiB |
| WebView2 GPU | 9728 | 76.104 MiB | 76.223 MiB |
| WebView2 network | 28464 | 12.029 MiB | 12.203 MiB |
| WebView2 storage | 25844 | 7.542 MiB | 7.629 MiB |
| WebView2 renderer | 35848 | 75.400 MiB | 76.137 MiB |
| WebView2 CDM broker | 29656 | 68.542 MiB | 68.637 MiB |
| WebView2 audio | 25260 | 8.153 MiB | 8.230 MiB |

Roles come only from Chromium type/subtype flags, not complete command lines,
URLs or profile contents. Individual peaks need not share a snapshot. The raw
counter report is retained locally with SHA256
`e89899674b6ff2f625d058ce35da7b1207b615edece59020c33c40887e88ca1a`.
Polling cannot see a helper created and exited entirely between snapshots.

This sample still exceeds the original 250 MiB private-memory limit. WebView2
alone exceeds that value at the aggregate peak. The earlier preview differs in
runtime version, process set, uptime and uncontrolled interaction; the numerical
difference is not a measured improvement or a diagnosis of a leak. Neither
sample measures the effect of the [hidden preset scan fix](settings-preset-scan.md).
Controlled latest-preview samples and the one-hour gaming test remain open.
