# Apple API failures and recovery

The `fix/apple-api-recovery` follow-up preserves a bounded HTTP status for failed
Apple metadata requests and mutations. Fixed messages distinguish rejected app
credentials, account authorization, rate limiting and service failures. They
point to the existing developer-token import or sign-out/sign-in controls in
Settings > Account. No failure automatically signs out, opens authorization,
deletes cached music or retries a write.

[Apple's response guidance](https://developer.apple.com/documentation/applemusicapi/handling-requests-and-responses)
identifies 401 as a developer-token problem and 403 as a media-user-token or
authentication problem. The current [official v3 SDK](https://js-cdn.music.apple.com/musickit/v3/musickit.js)
stores an HTTP response error's numeric status under `error.data.status`.
The inspected local UTF-8 copy has SHA256
`21908b72bdcdeea3ec91fa40b105d0b9a2ba07c2a3fe32ea77edcd26e735012c`.
That implementation detail can change; missing or unrecognized fields retain
generic guidance. Only integer statuses 400 through 599 cross the bridge.
Response bodies, headers, URLs, SDK messages and credentials are never added
to an error event or diagnostic log.

Songs now keeps its read error separately from playback errors and copies it
into the existing table error row. Cached songs remain usable. Retry clears
the old read error immediately, shows loading and preserves the queue. Late
responses from an older read, host or account remain ignored. Apple error
messages wrap while reserving room for Retry; controls retain their existing
placement. Spotify error-row behavior is unchanged. The new error state is
in memory only, with no settings, credential or cache-format migration.

## Evidence and limits

The Node status/privacy regression fails before the bridge fix, losing status
401. It then passes for library reads with and without IDs, page reads, playlist
creation/appends and Favorites writes, including malformed status fields.
Native tests verify status bounds and fixed messages. App regressions verify
cached library/queue/playback preservation and stale read rejection. A UI test
reproduces truncated recovery text before the wrap fix, then verifies full text,
the Retry hit target and its actual reload action.

The read-only Windows probe passes using restored protected authorization and
a fresh owned profile: 100 library songs and 10 catalog search results. It sends
only hide, metadata-read and shutdown commands, with no audio, popup, sign-out
or account mutation. The profile was removed after shutdown. An existing
Applifast process was left running. This does not explain the earlier isolated
rejections or establish revoked/expired-credential recovery against Apple's
live service. The maintainer separately confirmed ordinary desktop refresh and
audible playback in the installed preview, with revision not supplied.

The [native HTML comparison](review-api-recovery/index.html) uses identical
deterministic fixtures on baseline `bd361d8` and the candidate, with two cached
songs and rejected app/account, rate-limit and service errors in both themes
and window sizes. These captures do not revoke or alter real credentials.
Windows formatting, strict default/demo all-target Clippy, default/demo
all-target tests, doctests, strict demo Rustdoc, demo build, gettext consistency
and Node bridge/token-signer checks pass. Library suites pass 1,024 and 1,068
tests, with six opt-in checks ignored in each ordinary run. The standalone host
passes strict all-target Clippy, build and 18 tests with one opt-in test ignored;
both app and host credential-store round trips pass separately with dummy grants.
Four Flatpak metadata checks pass. Linux launcher fixtures lack Ruby/POSIX tools,
and one release-name fixture lacks Windows symlink privilege. Hosted environments
remain required for all-feature, documentation-site and Nix checks.

All 32 native captures were inspected and have matching dimensions with the
recovery row visible in the candidate. Existing translations remain unchanged.
The HTML's selectors and Before/After handlers pass a static Node VM check.
Browser rendering remains unverified after the earlier browser-client EPERM;
no alternative automation was used. Optimized package and hosted-CI results
accompany the feature PR. Fresh-account authorization, live credential rejection,
audible playback, rendered HTML, interactive motion and gaming acceptance remain open.
