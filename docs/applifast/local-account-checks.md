# Local Apple account checks

Windows check: October 22, 2026. These results cover the native playback host
with saved protected authorization. They do not establish fresh Apple sign-in,
desktop control interaction, a different PC or release readiness.

## Playlist creation and append

The maintainer explicitly authorized one **Applifast QA** playlist. The opt-in
Windows test used the production typed commands and unchanged MusicKit bridge
to create that playlist with an existing library song, append a second existing
library song, and read back both exact library resource IDs in the expected
order. It passed in 5.30 seconds (5.523 seconds including the driver). No catalog
substitution, favorite, playback, authorization-popup or sign-out command was sent.
The playlist remains in the account for inspection.

The test first completes playlist pagination and refuses an existing playlist
with that name or an unrecognized response. It uses only library identities.
Before either mutation it durably reserves a fixed local attempt marker and
writes an atomic phase receipt. A timeout or ambiguous response stops the test;
neither POST is automatically repeated. The ordinary guard regression verifies
exclusive reservation, incomplete-receipt rejection and atomic replacement.
Running the ignored check without explicit local permission rejects it before
starting the host.

The test is ignored by normal suites and compiled only for Windows tests. To
prepare another deliberately authorized check, a maintainer must inspect Apple
Music and the retained receipt first. Do not delete the marker simply to retry.
The test requires `APPLIFAST_ALLOW_QA_PLAYLIST_WRITE=create-one-qa-playlist`,
`APPLIFAST_QA_RECEIPT_DIR` pointing at this checkout's fixed
`.cache/apple-account-qa`, and a new child `LOCALAPPDATA` directory inside it.
Only the child test receives those environment variables. Its existing protected
grant is restored internally; no token or browser profile is copied.

Local receipts may contain the QA playlist ID. They stay ignored under `.cache`
and must not be committed or shipped. Disposable browser profiles can be removed
after their host exits, while retaining the attempt marker, receipt and sanitized
result. Ordinary application storage and network behavior are unchanged. This
test calls the existing Apple library playlist create/append/read endpoints.

The successful binary was built from `0e6aed49aa4cc886d0641e1686e3ab412d7b38a3`
plus the new test module. Its SHA-256 was
`b149102a6696b06fb25c5147651df00480f3bd9673cb89d308aa5eb590b9e49b`;
the fixture source SHA-256 was
`2b658b2f1f9b246f85ba2474b42ecef701c6107872e548ab553aa64e3f389585`.
The production bridge SHA-256 was
`625fd438b62a54ab19e1f975e76999b4081a02112c21bf3a778645e9c49de164`.
The final module declaration was moved after the account run without changing
the fixture or production code. Sanitized results and the private operation
receipt are retained locally, rather than publishing account identifiers.

An initial restricted-environment attempt failed during host startup in
0.04 seconds, before the mutation marker existed or any write command was sent.
With normal Windows credential-store access, the same binary first passed the
read-only host check, then passed the authorized write check. This distinguishes
that execution-environment failure from an Apple mutation failure.

## Reads and existing Favorites

The unchanged saved-account checks also passed with fresh owned browser profiles:
100 library songs, 10 catalog songs and both small/original-derived artwork paths.
The latest read-only run with the playlist-test binary took 5.43 seconds.
The separate Favorites check independently confirmed existing library and catalog
favorites before idempotently re-adding them, then verified the exact identities
and favorite state. It passed in 6.54 seconds on the preceding demo-test binary.
No song was newly favorited. See the [original Favorites evidence](favorites-write-evidence.md)
for that check's scope and history.

## Remaining local and external checks

Fresh authorization/cancel/retry, a new favorite chosen by the maintainer, normal
desktop playlist controls, restart/refresh confirmation, recovery and a long
listening session remain separate gates. Local profile isolation can exercise
saved-account paths, but it does not emulate a clean Windows installation.
Windows Sandbox was unavailable on this machine. Dad's independent installation
and Apple account are still useful acceptance evidence.

Resource measurements must include the app and its browser/GPU/helper processes,
identify the actual installed build, and verify paused/minimized state. Account
tests and builds must stop during the seven-minute resource sample. Readiness
and results remain tracked in the [release checklist](release-readiness.md).

## Automated validation

The final Windows source passes formatting, strict default/demo all-target
Clippy, default/demo all-target tests (1,041/1,089 library tests passed, nine
opt-in checks ignored in each ordinary suite), doctests, strict demo Rustdoc,
the demo build, gettext validation and Node bridge/token self-tests. The account
write above ran separately with explicit authorization. No credential format,
dependency or ordinary-interface change is included. Full-feature/platform,
documentation-site and Nix validation remain hosted CI gates on this source.
