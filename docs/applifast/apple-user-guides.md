# Apple Music documentation and current-preview recheck

The existing Jekyll guide structure now describes Windows Apple Music:
the landing page, overview, downloads, installation, everyday use, mini player,
privacy, file locations and connection overview. The hero/cards and guide routes
use the existing theme. The main/mini-player application interface is unchanged.
Current screenshots reuse approved native Apple demo captures, not account data.

Downloads identify the invited, unsigned x64 preview and its real prerequisites.
They no longer link to upstream Spotify installers as if those were Apple builds.
The version selector says Windows preview rather than claiming a released stable
Applifast version. Legacy MilkDrop, personal Spotify setup and connection/settings
details are labeled for contributors. MIT, Spotifast and Carmine Paolino credit
remain; source, edit and issue links point to the fork.

The tester, implementation and readiness pages now have Jekyll front matter so
their linked HTML pages can render. The existing capability history no longer
claims Favorites are still unimplemented. No release version, signing key,
domain, production deployment, updater, tag or public download is activated.
The inherited site URL remains a publication-setup gate; this PR does not claim
ownership of the upstream domain.

Rendered review also found the production-only upstream analytics include. The
fork removes it. CI preview builds remain enabled; automatic site publication
remains disabled. App network behavior is unchanged.

CI retains the actual Jekyll base/candidate output for seven days in a
`docs-preview-<sha>` artifact. This supports HTML/browser inspection without
publishing either site. It uses the existing upload action and read-only workflow
permissions. Local Ruby/Jekyll is unavailable, and Docker is not running. The
actual candidate and base site builds pass on `55c0450` in
[CI run 39998380866](https://github.com/jackkayser2005/applifast/actions/runs/39998380866).
The [HTML comparison](review-user-guides/index.html) contains 56 actual rendered
Windows Chrome captures: six pages and open navigation menus, both themes and
normal/narrow widths. Their files, dimensions and hashes are recorded. Page
captures assert matching theme, local image loading and no horizontal overflow.
The three explicit HTML routes return HTTP 200; the tester-guide link opens its
destination. An initial click check inspected the URL before navigation finished;
waiting for the destination fixes that QA timing error without a site change.

All external browser requests were blocked before the matching captures,
including inherited baseline analytics, remote fonts, counts and promotional
images. The first unblocked batch was rejected by automatic approval review;
the blocked-request alternative passed. These limits are in the comparison.
Large videos were omitted from the local review copy. This is rendered website
evidence, not native-app or live external-service acceptance.

## Current installed application evidence

The optimized installed `8da79db` app has the expected executable hash. The
[two resource samples](preview-resource-check.md) include every observed helper.
The paused/minimized run measures 531.008 MiB peak private memory and 0.0439%
average total CPU. The observed paused CPU target passes; memory fails the
original 250 MiB target. The earlier mixed-state sample is retained with its QA
wrapper limitation. Task Manager's native memory row is not the aggregate total.
A separate post-sample counter snapshot shows 42.598 MiB aggregate private
working set, plus shared resident pages. This does not replace the five-minute
private-byte measurement or establish sustained working-set usage.

The separate [listening evidence](preview-listening-check.json) confirms:

- 73 ordinary Apple behavior tests pass, with nine opt-in checks ignored.
  They cover pagination/stale answers, host retry, queue retention, Favorites,
  playlist confirmation and account-bound restoration. A restricted Windows
  run blocked atomic file replacements; normal access with a disposable
  workspace temp directory passes without weakening any assertion.
- Native synthetic cookie clearing, cleanup-before-authorization ordering and
  an isolated dummy Credential Manager round trip pass. These do not sign out
  the listener or establish real Apple revocation handling.
- The current saved-account native transport check passes in 52.095 seconds:
  library/catalog shelves and search, muted play/seek/pause/resume, saved-position
  restoration, duplicate removal, Play next and a 1,001-item context.
- Two complete muted tracks pass in 188.332 seconds. Both declare 90.013 seconds,
  reach actual position 90.00 and play for 90.15 seconds before natural completion.
  Manual-before-context duplicate ordering then starts the separate occurrence
  and pauses. No seeking or account write is used in this check.

Both native account runs use separate owned browser profiles, restore existing
protected authorization, and leave the normal player paused. They stop if it
resumes. No playlist, new Favorite or sign-out command is sent. Their completed
test profiles are disposable; the earlier one-time playlist-write guard remains.

The application source is unchanged from `8da79db`. That feature and the prior
integration head each pass all seven hosted jobs. This documentation/workflow
follow-up passes the actual base/candidate site build; the remaining required
hosted jobs on its final head and maintainer review remain separate.

## Still open

Fresh-PC Apple authorization/cancel/retry, real expiry/revocation and account
switching, live network loss, output-device changes, audible main/mini/tray
recreation, new Favorites and desktop write interactions remain acceptance work.
Muted saved-account tests do not close them. The original memory gate, controlled
playing sample and one-hour gaming session remain open too.

Source integration, beta metadata, final checks, signing/token-renewal decisions
and verified publication follow the full [release checklist](release-readiness.md).
