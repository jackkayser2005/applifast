# Applifast Windows preview installer

Baseline: `8b09739065f1b032e512e9be0a26322a9841ddc2` (PR #34).
Windows x64 validation: October 10, 2026.

## Installed behavior

The upstream installer still identifies Spotifast and registers Spotify links.
The fork now has a separate `packaging/windows/applifast.iss` template. It
packages exactly the six validated portable files, with no wildcard sources:
Applifast.exe, BUILD.txt, LICENSE, README.txt, TESTING.md and developer-token.txt.
Inno Setup adds its own uninstaller. No `.p8`, user grant, browser profile or
account cache enters the installer.

The stable AppId is `Applifast.Windows`, separate from the upstream GUID.
Inno uses AppId to identify upgrades and the uninstall registry entry:
[AppId documentation](https://jrsoftware.org/ishelp/topic_setup_appid.htm).
The installer uses the current user and `%LOCALAPPDATA%\Programs\Applifast`,
with a Start shortcut and optional desktop shortcut. Its shortcut AppUserModelID
matches the app's `applifast` window identity. No URL handlers, startup task,
forced application shutdown, elevation or WebView2 download are added.

Running a new installer after quitting the old app replaces program files,
including the bundled app token. The existing protected-token preference and
subscriber authorization behavior are unchanged. Uninstall removes installed
program files, shortcuts and its uninstall entry. Settings, metadata/queue
caches, browser profile, credentials and unrelated files are preserved. Sign
out in the app first to clear its subscriber authorization and account snapshot.

The setup display name includes the source revision and says development
preview. Its numeric version comes from the executable's inherited `0.12.0`
file version. The executable, installer icon, release versioning and signing
still have separate pending release work. The installer and application are
unsigned; SHA256 checksums detect changed files, not publisher identity.

## Building

Use the existing packaging command with an installed Inno Setup compiler:

```powershell
./packaging/windows/applifast-preview.ps1 `
  -DeveloperToken .secrets/apple-music/developer-token.txt `
  -SetupCompiler 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
```

Omit `-SetupCompiler` for a ZIP only. The command validates the token locally,
checks that installer inputs contain an x64 PE executable, preserves existing
outputs, builds the six-file ZIP and optionally compiles the installer with its
own checksum. A compiler failure reports failure and preserves the valid ZIP.
Apple verifies the token's signature and access at runtime. The `.p8` signing
key must never be supplied or distributed.

Local validation uses official Inno Setup 6.7.3. Its download SHA256 is
`9c73c3bae7ed48d44112a0f48e66742c00090bdb5bef71d9d3c056c66e97b732`, matching
the immutable release asset digest, with a valid Pyrsys B.V. Authenticode
signature. The compiler was installed in portable mode under ignored workspace
cache, without adding an installed-program entry. See
[official downloads](https://jrsoftware.org/isdl.php),
[verification](https://jrsoftware.org/isdl-verify.php) and
[portable-mode documentation](https://jrsoftware.org/ishelp/topic_technotes.htm).
The compiler is a build tool and is not shipped with Applifast.

## Checks and remaining gates

The ordinary package regression uses dummy JWTs. The installer regression
compiles the real template, then compiles a QA variant with a unique uninstall
identity and no launch action. Only that QA variant is installed in a disposable
workspace directory. It checks the exact payload, checksum, per-user uninstall
entry, in-place guide/token upgrade, removal of owned files/registration and
preservation of unrelated files. It creates uniquely named QA Start/desktop
shortcuts, verifies their targets and checks their removal. The app never
launches and no actual listener
credentials or profile are accessed. QA uninstall and directory cleanup are
part of the check.

The CI definition now includes standalone Windows host format/Clippy/tests,
the isolated native credential-store round trip, bridge/signer checks and x64
preview/package/install regressions. It does not use real Apple secrets. The
initial API snapshot showed zero registered workflows and no runs for the
preceding PR head. PR #35 subsequently started CI and Packaging runs. Hosted
acceptance still requires completed jobs on the final head, not merely a changed
definition or a started run.

On `afb2a1e`, [CI run 38079141932](https://github.com/jackkayser2005/applifast/actions/runs/38079141932)
passed the documentation-site build and Linux packaging regression steps while
compilation and Nix jobs were still running. This records that observed head's
progress, not final-head CI acceptance.

The first Nix job on `d19fdec` failed with a vendor-staging hash mismatch after
the earlier Apple dependency changes. `flake.nix` now uses Nix's reported actual
hash, `sha256-JnLrhJTGuAskwUtKbmrLtQRDpGh3PytrTGsa6KLRW7o=`. No lockfile,
dependency version or Windows code changes accompany that refresh. The new
head's Nix build must still verify it; the failure is not treated as a pass.

Local installer regression passed: production-template compilation, x64 input
validation, missing compiler rejection, compiler-failure ZIP preservation,
existing-output protection, checksum, six installed file hashes, per-user QA
registration, Start/desktop shortcut targets, token/guide upgrade, uninstall and
preservation of unrelated files. The QA shortcuts and registry entry were
removed and the successful QA directory was cleaned up.

Standalone host formatting, strict all-target Clippy, 13 ordinary tests, the
isolated native credential-store round trip and the build passed. App
formatting, strict default/demo all-target Clippy, 1,009 default and 1,043 demo
library tests (four opt-in tests ignored in each), binary/integration suites,
default doctests, strict demo Rustdoc, the demo build, gettext and Node bridge/
signer checks passed. The package fixtures use dummy JWTs only. Packaging
PowerShell syntax, CI YAML/platform guards and README/tester/evidence links pass.
Native installer-wizard captures and visual approval, clean-user sign-in,
actual application startup from installed shortcuts, real-account restart,
gaming/resource measurements, ARM runtime, Linux/macOS, optional projectM,
Jekyll and Nix remain unverified. No release tag or public artifact is published
by this change. The egui interface does not change.
