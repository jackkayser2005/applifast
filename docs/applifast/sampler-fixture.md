# Windows sampler fixture lifetime

The Windows x64 job in [CI run 38811155358](https://github.com/jackkayser2005/applifast/actions/runs/38811155358)
failed after its app tests, host tests, bridge checks and dummy credential-store
round trips passed. The sampler fixture used a 30-second sleeping process.
Cold initialization of the sampler's `Add-Type` process inventory outlasted that
process, so `GetProcessById` failed before the first measurement.

The fixture now waits for its parent to close a redirected input pipe. The
existing `finally` block closes the pipe, waits up to five seconds and kills
only the owned helper if necessary. It remains hidden and needs no dependency,
account access or application process. The sampler and its acceptance criteria
are unchanged.

Local Windows verification on October 15, 2026:

- The ordinary real-counter sampler self-check passes.
- Injecting a 35-second delay before the inventory's first `Add-Type` reproduces
  the old fixture's missing-process error. The updated fixture passes the same
  delayed check, including process identity and per-process memory accounting.
- PowerShell parsing and `git diff --check` pass.

The delay is an isolated test-session override, not a production sleep or retry.
Hosted checks on the new source still need to pass before integration. These
short dummy-process samples do not verify playback state, the five-minute
resource budget or the gaming soak. The existing private installer is unchanged.
