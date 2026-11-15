# plan-k1

## Goal

Bring the Linkuistics Herdr fork up to date with upstream, release the updated
fork through the maintained Homebrew formula, and upgrade the local install.

## Context

The human also reports that switching spaces removes the two most recent lines
from a pane after an agent has exited. Check whether this persists after the
sync and investigate with an isolated reproduction when the trigger is clear.

## Done when

- The fork includes current upstream master and preserves its three patches.
- The updated tip has passed the documented checks, with any baseline failures
  verified against pure upstream rather than waived speculatively.
- The fork bookmarks and Homebrew formula are published to the human's forks.
- The local Homebrew install reports the new fork version.
- The reported history loss has been checked or its unresolved trigger recorded.

## Decisions (running log)

- Requirements are settled by the human's requests and `FORK.md`; no full
  requirements interview is needed. Execute this bounded maintenance work
  directly rather than cutting a requirements/design/planning chain.
- Follow `FORK.md`: use jj for mutations, rebase the existing fork patches onto
  `master@upstream`, and release through the Homebrew formula rather than an
  upstream release workflow or GitHub release.
- Preserve existing user state. The initial working change contains only the
  two newly scaffolded Grove files; the fork bookmark is `f5f2cd15` and the
  installed version is `0.9.5-linkuistics.1`.
- `~/Development/homebrew-taps` does not exist at the documented location.
  Locate the maintained source checkout before editing the formula.
- Upstream was fetched at `ee5bfb5041bb3e4a49d591c72308aae3c81ecdde`;
  all eight fork changes rebased onto it without conflicts. Validate this tip
  before publishing.
- The maintained tap source is now
  `/Users/antony/Development/Delivery/homebrew-taps`. Correct `FORK.md` so
  future maintenance uses that checkout.
- `just check` passed formatting and clippy, then stopped at the documented
  `api_ping::pane_info_and_subscriptions_expose_done_agent_status` timeout
  (3,839 tests passed before fail-fast). Run the remaining checks with this
  known upstream baseline test excluded; retain the failing run as evidence.
- The graph index could not start because daemon coordination could not be
  verified safely. Use direct source reads for the history-loss investigation.
- Initially interpret “switch spaces” as switching Herdr workspaces while
  awaiting clarification. Reproduce only in the named disposable session
  `repro-history-20261116-0044`, attached through the created outer pane `w32:p2`.
- With the known timeout excluded, all 3,974 Rust tests passed, as did 150
  maintenance tests, six UI architecture tests, all integration asset suites,
  and seven documentation contract tests. Logs are under
  `/var/tmp/herdr-sync-20261116-*.log`.
- Windows lint cannot run because this machine has no configured Windows SDK.
  The same command fails with the same missing-SDK message in the separate
  pure-upstream workspace `/var/tmp/herdr-upstream-validation-20261116` at
  `ee5bfb50`. Do not install an SDK or accept its licence as part of this sync.
- Two workspace round trips preserved the numbered shell history exactly on
  installed `0.9.5-linkuistics.1`. Next test an actual Codex startup and exit
  without submitting a prompt or spending agent tokens.
- Proposed maintenance subject `chore: sync linkuistics fork with upstream`
  and formula subject `linkuistics-herdr v0.9.5-linkuistics.2` for the required
  human alignment before sealing and publishing new changes.
- The human confirmed that “spaces” means Herdr workspaces and approved both
  proposed commit messages. Proceed with publication after the isolated check.
- Release the validated rebased source at
  `58373aed87cbf8e97d8017a1d9c89a7f0941acab`; the final maintenance commit
  records installation and task retirement afterward. Its documentation-only
  changes do not alter the released code or require rebuilding it.
- The disposable outer pane and its test workspaces closed before the
  exited-agent comparison completed. Preserve that run as inconclusive and
  repeat in a fresh disposable session after installation.
- The source and formula were published, but the Homebrew build failed while
  Zig fetched `uucode` with HTTP 400. The existing install was restored by
  Homebrew. Investigate the download before retrying the upgrade.
- The human requested a project `mise.toml` to manage Zig. Pin `zig` to
  `0.16.0`, use mise for local builds, and preserve Rust's existing
  `rust-toolchain.toml`. Include this tooling configuration in the approved
  maintenance change.
- The hidden standalone restart command is `herdr server live-handoff`.
  Specify `--import-exe /opt/homebrew/bin/herdr` after a Homebrew upgrade so
  the running server imports the newly installed binary. Instructions were
  supplied; replacing the default server was not requested or performed.
- Both Homebrew and mise Zig 0.16.0 reproduce HTTP 400 for the pinned `uucode`
  archive. Curl downloads it successfully, and Zig's local archive import
  confirms the exact manifest package hash. Passing a populated cache through
  the shell did not survive Homebrew's build environment. Add the archive as
  a SHA-256 checked Homebrew resource and import it into Zig's build cache
  before Cargo, preserving the vendored dependency and released source.
- The single-resource retry imported `uucode` successfully, then exposed the
  same download failure for other native packages. Extend the checksum-verified
  resource approach to the observed eight-package set: translate-c, uucode,
  ghostty-themes, highway, zlib, wuffs, pixels, and transitive aro. Preserve
  the original URLs and Zig content hashes; do not fetch unrelated lazy GUI
  dependencies or change vendored source.
- Homebrew installed `0.9.5-linkuistics.2` successfully after importing all
  eight resources. Both `/opt/homebrew/bin/herdr --version` and `brew list
  --versions linkuistics-herdr` confirm it. Repeat the history check in new
  session `repro-history-20261116-0123`, using only created outer pane `w32:p3`.
- Fresh validation under mise selected Zig 0.16.0 and Cargo 1.96.1. The full
  `just check` stopped at the same known API timeout; with that case excluded,
  3,974 Rust tests, 150 maintenance tests, six UI architecture tests and all
  integration asset suites passed. Seven documentation contract tests passed
  separately. Windows lint still reports the previously verified missing SDK.
  Homebrew's installed formula test passed.
- On installed `0.9.5-linkuistics.2`, plain shell history survived a workspace
  round trip. Codex 0.169.0 was started with `--no-daemon --no-alt-screen -s
  read-only`, its hook trust prompt was skipped, and it exited without a prompt
  or paid tokens. After confirming the shell was foreground, a workspace round
  trip preserved the complete captured history. Two new recent output lines
  also survived a further round trip, with both recent and visible snapshots
  identical. No history-loss fix is established: the trigger after an actual
  agent work session remains unverified. Evidence is under
  `/var/tmp/herdr-history-20261116-0123`.
- The disposable session was stopped and deleted, its logs copied into the
  evidence directory, and only created outer pane `w32:p3` was closed. Session
  listing now shows only the default server, and parent pane `w32:p1` remains.
- The Obsidian Development Environment note now records Herdr's mise Zig pin
  and Rust selection. Read-only `mac-check` exited 10: setup and global tools
  passed; it reports unsaved tracked configuration, a changed Claude settings
  receipt, and manual CCC backup review. Leave those separate machine tasks
  with their owner; do not publish dotfile history or change backup settings.

## Notes
