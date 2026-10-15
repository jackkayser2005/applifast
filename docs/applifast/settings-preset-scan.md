# Apple Settings avoids hidden preset scans

Apple playback does not expose MilkDrop. Opening Settings nevertheless refreshed
its retained preset index before checking whether that section was available.
The refresh recursively enumerates `.milk` files and retains their paths, even
when searching Settings for an unrelated Apple control.

Settings now refreshes this index only in retained Spotify mode. Apple mode
keeps its existing supported controls and decorative Ambient Pulse. There is
no layout, setting, storage-format, dependency or network change. Existing
preset files are preserved. This removes unnecessary work; it is not a measured
startup, memory-budget or gaming-performance improvement.

## Verification

The regression creates two disposable presets, including one in a nested folder,
then draws actual Settings frames with no filter, a MilkDrop search and a Reduce
motion search. Apple mode must retain no preset paths. Switching the sample app
back to Spotify mode must discover both files, preserving that integration.
Before the fix the first Apple frame discovers both presets and fails the test.
This fixture uses no account, authorization, audio or network request.

After the fix the regression passes. Windows formatting, strict default/demo
all-target Clippy, default/demo all-target tests, doctests, strict demo Rustdoc,
demo build, gettext consistency and Node bridge/signer checks pass. Library
suites pass 1,027 default and 1,073 demo tests, with six opt-in tests ignored per
ordinary suite. Fifteen translation files update source references only; their
message semantics are unchanged. Git Bash required execution outside the
sandbox because its named-object initialization was denied there.

Four fresh native Settings captures at 1100×760 and 760×760 in dark/light themes
are pixel-identical to the matching `64db4c3` captures in the
[existing overall comparison](review-integration/index.html). There is no new
visible interface scope. These stills do not establish live motion or browser
rendering. All-feature/platform, site and Nix checks on this follow-up remain
separate from local Windows checks.

The integration baseline is `64db4c3`. Its seven hosted CI jobs pass, including
Windows x64/ARM, Linux/macOS, strict quality checks, Jekyll and Nix. Those checks
do not establish this follow-up's final head or Windows account acceptance.

Fresh-PC sign-in, live listening/account writes, interactive motion, narrow-queue
responsiveness and the original resource/gaming gates remain in
[release readiness](release-readiness.md). The muted native account check was
deferred when its process guard found the maintainer's listening app running.
The guard sent no playback or account command.
