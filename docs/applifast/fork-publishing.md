# Fork publishing boundary

The inherited production documentation still carries `spotifast.rocks` in its
domain configuration and upstream site metadata. The legacy tagged release
pipeline names Spotifast artifacts and builds macOS/Linux packages, whose Apple
playback engines do not exist. These are retained upstream assets, not an
Applifast Windows release path.

Before the port moves to `main`, publication is bounded as follows:

- Production Docs build/deploy jobs run only in `crmne/spotifast`. A push to this
  fork's main branch cannot configure or deploy the inherited Pages site.
- The legacy tagged release graph and manual Flatpak-from-release job run only
  in the upstream repository. They do not create Applifast downloads.
- Reusable native-package publication requires the upstream repository, an
  explicit publish request and a nonempty version. The pinned upstream fixture
  and its installation checks remain available to forks with publication off.
- The external native-package workflow receives only the four publishing
  secrets already declared by this wrapper. It no longer inherits every repo
  secret, including any future Apple app token.
- Normal CI retains strict quality checks, four platform test jobs, the Jekyll
  build and Nix. The manual Windows preview workflow stays available and retains
  read-only repository-content permission. Its app-token setup and first hosted
  artifact run remain separate acceptance steps.

GitHub documents [repository-scoped job conditions](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-jobs-with-conditions)
and that the reusable workflow's [GitHub context belongs to its caller](https://docs.github.com/en/enterprise-cloud%40latest/actions/reference/workflows-and-actions/reusing-workflow-configurations).
The conditions deliberately name the upstream repository. Enabling a public
Applifast release or website requires a reviewed fork-specific workflow and
identity, rather than reusing those publishing jobs by mistake.

## Verification and remaining work

`ruby packaging/test-fork-workflows.rb` parses the workflows with Ruby's standard
YAML library. It checks every legacy publishing entry point, the reusable
publisher's opt-in/version conditions and secret scope, and retained CI/platform
and manual-preview paths. Negative cases remove each guard or introduce an OR
bypass, and reject unrestricted secret inheritance. The quality job runs this
check before Clippy. Ruby is already used by the existing packaging/site checks;
the application gains no dependency.

There is no local Ruby runtime in the current Windows environment. Hosted
validation of this regression and the changed workflows is required before
integration. This source change has not dispatched a production deployment,
configured a domain/secret, pushed a release tag or published an artifact.
It changes no application UI, playback, storage, credential or network behavior.

The Windows beta still needs its own reviewed version/notes, final checks,
signing decision, token delivery/renewal, verified artifacts and download links.
Its [original release gates](release-readiness.md) remain open; these guards do
not waive runtime, visual or performance acceptance. The qualified `1742628`
private installer remains unchanged. The separate hidden-host memory candidate
is not included in this branch.
