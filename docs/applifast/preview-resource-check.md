# Current Windows preview resource check

The installed optimized `8da79db` preview was measured on the maintainer's
16-logical-processor Windows PC. The executable SHA-256 is
`017d483c90c35adfdf4fdcd791d21c2c452c7a5802de632f09acb7542de48def`.
The [sanitized evidence](preview-resource-check.json) retains aggregate counts,
per-process groups and hashes of the raw local reports. It contains no song
metadata, authorization, process command lines or account identifiers.

Each run used a two-minute warm-up and a five-minute sample. No builds or tests
ran during either sample. Only process counters, read-only playback status and
window visibility were queried; no playback or window commands were sent.

| Run | Peak private MiB | Average total CPU | State qualification |
| --- | ---: | ---: | --- |
| Initially playing/minimized | 534.820 | 0.0361% | State became unverified after 151 seconds of the complete run |
| Paused/minimized | 531.008 | 0.0439% | All 28 status/visibility observations confirmed paused and minimized |

The first QA wrapper mistakenly excluded `paused` from its status whitelist.
It recorded unreadable status from 151 seconds onward. A subsequent independent
read correctly reported paused. The wrapper is corrected for future runs, but
the original observations are preserved rather than rewritten. That run is
not a controlled playing-state acceptance pass.

The paused sample contains 284 counter observations over 300.179 measured
seconds. Its peak comprises **278.086 MiB native** and **252.922 MiB across eight
WebView2 processes**. Process accounting was complete for the observed tree.
Processes that start and exit entirely between snapshots remain unobservable.

The normal executable's description is **Apple Music for Windows**, matching
the label in Task Manager. A Task Manager memory row and aggregate private bytes
are different metrics. The screenshot is not a complete private-memory total.

A separate [post-sample working-set snapshot](preview-working-set-check.json)
uses Windows process performance counters and verifies the same nine process
identities. Private working set is 5.438 MiB native and 37.160 MiB across WebView2,
or 42.598 MiB combined. Total working set, including shared pages, sums to
358.047 MiB; that sum can count shared pages in several processes. Private bytes
at that instant sum to 530.782 MiB. These are three different counters, not
conflicting estimates of one quantity. The working-set snapshot is not a second
five-minute benchmark, a unique-physical-memory total or a before/after saving.

The observed paused CPU result is below the original 0.2% target. State and
minimization were checked at 15-second intervals, not continuously through the
SDK. **The 250 MiB aggregate private-memory gate fails.** Neither sample proves
the one-hour gaming session or a before/after saving. The older installed
measurements used different builds and states and are not matched baselines.

Next, collect a controlled playing sample, profile retained native artwork and
renderer allocations, and compare equivalent fresh-start/library/main/mini/tray
states before changing memory policy. Count every WebView2 helper. Do not trade
away playback correctness or relabel the original limit.
