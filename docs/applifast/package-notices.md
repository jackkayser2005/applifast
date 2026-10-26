# Notices in Windows previews

The Windows packager previously copied only the root MIT license. The source
tree also carries notices for the bundled Inter and Noto Emoji fonts and Lucide
icons. Their complete existing texts now follow the app's MIT license in the
packaged `LICENSE` file, with component labels. The source MIT license is
unchanged. The installer uses that same payload file on its license page.

The six-file installer/ZIP allowlist, protected credentials, token validation,
source/checksum manifest and installation identities stay unchanged. No new
dependency or network request is added. This copies existing notices; it does
not claim a complete third-party dependency-license audit.

The archive regression fails before the fix because the Inter notice is absent.
It passes afterward only when every full source notice, including the root MIT
text, is present in the archived file. Dummy-token packaging tests retain their
existing rejection, checksum and allowlist checks. Native installer fixtures
also pass compilation, six-file payload/checksum, per-user install, shortcuts,
upgrade and uninstall checks. They use a unique QA registration and dummy token,
preserve unrelated files and never launch the app. Native wizard screenshots
remain an open visual check from the installer milestone.
