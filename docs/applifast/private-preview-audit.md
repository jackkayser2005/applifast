# Private preview credential and package audit

Checked October 12, 2026, at documentation revision `1410e27`, for the
optimized `4f47613` Windows preview. This is a bounded credential-hygiene check,
not a general security assessment or public-release approval.

## Source and historical coverage

The integration checkout normally fetches one branch. The audit explicitly
fetched all published origin branches without changing the working tree, then
verified that all 50 published branch tips matched their remote-tracking refs.
It inspected every blob reachable from local refs and reflogs: 10,628 objects,
5,904 blobs and 596,409,256 raw blob bytes. Historical changed filenames were
checked separately so identical content under different names was not missed.

Results:

- No historical `.secrets/` paths, `.p8` filenames, `developer-token.txt` or
  `music-user-token` files were found in that scope.
- No complete PEM private-key blocks were found, including escaped line breaks.
- No MusicKit developer-JWT candidates were found. The check recognized ES256,
  ten-character key/team IDs, token dates and a nonzero 64-byte signature. It
  excluded deliberately invalid all-zero signatures used in tests.
- Ignore checks pass for the local secret directory, nested `.p8` files and
  developer-token files. The original checkout's actual key/token paths are
  ignored too, and neither checkout tracks those files.

The temporary scanner used synthetic positive and negative assertions before
reading repository objects. It retained only counts, paths and detection types,
never token or key contents. Its sanitized local report is
`.cache/release-secret-audit.json`; the local helper is
`.cache/check-release-secrets.py`. The audit did not open the local signing-key
file or access Credential Manager.

## Download coverage

The `4f47613` ZIP contains exactly six entries: `Applifast.exe`, `BUILD.txt`,
`LICENSE`, `README.txt`, `TESTING.md` and `developer-token.txt`. The same pattern
checks found no private-key blocks or developer-token candidates outside the
declared companion file. That companion contains the intended app JWT, restricted
to `https://applifast.invalid`. It is not a listener authorization token.

The earlier payload, manifest, executable architecture, checksum and default-
feature help checks pass. Dummy-token packaging and isolated per-user install,
shortcut, upgrade and uninstall fixtures pass without launching the app.

## Limits and next gate

This pattern check cannot prove the absence of opaque listener grants, encoded
or encrypted payloads, credentials visible only in images, or remote history
that is no longer reachable. It does not verify Apple's signature acceptance,
token/key revocation, clean-PC authorization or account-data clearing.

Repeat the source/history and final-payload checks on the actual release
candidate before publication. The current preview remains unsigned, its app
token expires November 6 at 17:47 UTC, and it has no automatic token renewal.
The [release checklist](release-readiness.md) retains those separate gates.
