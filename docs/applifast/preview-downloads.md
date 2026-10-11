# Windows preview downloads

Invited testers can receive an installer or portable ZIP without installing Rust
or running build commands. The local packaging command remains available in the
[installer guide](windows-installer-evidence.md). The **Windows preview downloads**
workflow prepares the same six-file payload from an optimized Windows x64 build.
It uses default features, so demo commands are unavailable in that download.

## Maintainer setup

1. Merge the workflow and its prerequisite integration PRs into the default
   branch. GitHub requires a manually dispatched workflow to exist there first.
2. In the repository's Settings > Secrets and variables > Actions, add
   `APPLE_MUSIC_DEVELOPER_TOKEN`. Its value is the signed developer JWT generated
   locally for `https://applifast.invalid`, not the `.p8` signing key, an Apple
   password or a subscriber token. The packaging validator checks dates, size,
   format and the exact origin; Apple verifies its signature and access.
3. After the selected source commit's required CI checks pass, choose Actions >
   Windows preview downloads > Run workflow and select that trusted branch.
   Only maintainers with repository write access should dispatch these builds.
4. Wait for the complete run to pass. Download its `applifast-windows-x64-<SHA>`
   artifact and extract the outer GitHub artifact archive. Share the contained
   `applifast-preview-<revision>.zip` or `-setup.exe` and its matching `.sha256`.
   Signed-in users with repository read access can download Actions artifacts;
   this is not a permanent, anonymous public download page.

[GitHub's manual workflow instructions](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)
describe dispatch and branch selection. Artifacts are retained for 14 days,
subject to repository policy. Token expiry is independent of artifact retention;
check `BUILD.txt` before sharing and renew the Actions secret when needed.
The app token is deliberately included in the six-file payload so subscribers
can click Sign in with Apple. It does not grant access to anyone's library.

## What the workflow verifies

The job builds the app and token-checking probe with locked dependency files,
without restored compiler outputs. It verifies the x64 PE architecture,
Applifast CLI identity and absence of a dynamic MSVC runtime dependency, then
runs the existing dummy-token package and isolated installer regressions against
that optimized executable. The actual app JWT reaches only the packaging step
through the Actions secret. Its temporary input file is removed, and only the
ZIP, installer and their checksum files are uploaded. No signing key, listener
grant, browser profile, account cache or diagnostic logs are uploaded.

The workflow runs only on manual dispatch. It does not create a tag or GitHub
Release, enable updates, change the inherited release workflow, or qualify a
build for beta acceptance. App and installer remain unsigned development
previews with inherited version `0.12.0`. SHA256 checksums detect file changes;
they do not authenticate the publisher. Fresh-user sign-in, actual listening,
favorites/playlist writes and the integrated resource/gaming checks remain
required before a public beta. Linux/macOS Apple playback is unsupported.

The workflow's first hosted run, using a configured valid developer JWT, remains
an acceptance check. YAML validation and local package tests do not substitute
for that run.
