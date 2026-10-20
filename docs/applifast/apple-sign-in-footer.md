# Apple sign-in affiliation wording

The signed-out Apple screen previously retained “not affiliated with Spotify”
under its Apple Music sign-in card. It now names Apple. Retained Spotify mode
keeps its previous wording. The version number, layout, controls, window sizing
and upstream credit remain unchanged.

The regression renders the actual login view in both service modes with no
credential restoration, media controls or tray. It checks the painted footer
after the initial layout frames, including the absence of the other service's
disclaimer. The check fails against the original Apple footer.

The [native comparison](review-apple-footer/index.html) pairs signed-out Apple
demo captures in dark/light themes at 1100 by 760 and 760 by 760. The baseline
is application source `3508c61`, using the existing profiler with SHA256
`f25070e9feab740ee7eb60b5a9e669f4bf20fbb9840433b5f14ed3b0d89f2ec1`.
Both phases share isolated demo directories, a two-second capture delay and
neutral pointer position. No account connection or audio is started.

All four candidate captures were visually inspected. Pixel comparisons find
changes only in the footer, within vertical pixels 725 through 734. The
[capture manifest](review-apple-footer/captures.json) records exact executable,
source and PNG hashes. All eight theme/size/phase selector combinations pass
the static HTML check. Subsequent [browser verification](preview-verification.md)
also passes all eight rendered states on source `50024ce`.

Local Windows formatting, strict default/demo all-target Clippy, all-target
tests (1,032/1,078 library tests, eight opt-in checks ignored per suite),
doctests, strict demo Rustdoc, demo/profiler builds, gettext and Node bridge/token
checks pass. The full localization integration test enforces the new entries.
The POSIX launcher fixtures cannot run in this Windows environment because
their Unix executables are unavailable; Linux CI, Ruby/site and all-feature
platform checks remain required on the PR source. Fresh-PC authorization and
the full product identity audit remain release gates.

This wording correction introduces no storage or credential changes, network
endpoint, telemetry, audio cache or additional dependency. All fourteen complete
translation catalogs retain the Spotify entry and add the Apple disclaimer,
preserving the version placeholder. Catalog completeness is enforced by the
existing localization integration test.
