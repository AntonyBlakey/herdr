# upstream-base-k2

## Goal

Stand up `linkuistics-fork-next` on upstream `master` tip and **prove it green
before a single fork patch lands**, so that every later failure in this grove is
attributable to the port rather than to upstream.

## Context

The working copy you inherit sits on the **old** line: the `requirements` commit
is a child of `linkuistics-fork`'s head (`085bcf89`), because that is where the
driver scaffolded `.grove/`. Your first act moves it.

```
jj rebase -r <the requirements commit> -d master@upstream
jj bookmark create linkuistics-fork-next -r @
```

That rebase is clean — the commit touches only `.grove/*.md`, and nothing
upstream has that path. It will look alarming (925 files change in the working
copy as the base swaps from v0.7.5-era fork to current upstream); that is the
point of the leaf.

**Do not move, delete, or push `linkuistics-fork`.** It is the specification the
next three leaves read.

Upstream tip at the time of writing is `9e9bc8a1 feat: manage multiple ssh
machines from one client (#3670)`. If upstream has moved on, take the newer tip
and record which commit you actually based on — the later leaves need to know.

## Done when

- `linkuistics-fork-next` exists, based on `master@upstream`, carrying `.grove/`
  and nothing else of the fork's.
- `just check` has been **run** and its result recorded in this file. Green is
  the expected outcome. If it is red, the failures are upstream's, not yours:
  record them verbatim here so the next three leaves can tell a pre-existing
  failure from one they caused, and **do not fix them** — that would be work this
  leaf was not cut for.
- The binary builds and starts. `env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH cargo run -- --help`
  is enough; you are proving the toolchain, not the app.

## Notes

- Vendored libghostty-vt needs **Zig 0.15.2** specifically — a newer Zig on
  `PATH` will fail the build. `CLAUDE.md` covers this.
- `just check` also runs maintenance tests asserting that local vendor patches
  are indexed in `vendor/libghostty-vt.patches.md` and reverse-apply cleanly.
  The fork adds **no** vendor patches of its own, so anything failing there is
  upstream's state, not something the port introduced.
- Upstream moved `vendor/portable-pty.patches.md` (+45) in the last two weeks;
  expect vendor-adjacent churn to be unfamiliar.
- Running a debug build from inside an existing herdr session needs the socket
  overrides cleared, as above, or you will talk to the installed stable server.

## Decisions (running log)

- **Based on `e366a05f docs: publish preview documentation`**, not the
  `9e9bc8a1` this file was written against. `jj git fetch --remote upstream` at
  session start moved `master@upstream` forward by one commit
  (`9e9bc8a1 feat: manage multiple ssh machines from one client (#3670)` is now
  its parent). Later leaves base on `e366a05f`.
- **Rebased with `jj rebase -s olxtvzox -d master@upstream`, not `-r`.** `-r`
  re-parents the moved commit's children onto its *former* parents, which would
  have stranded the empty working-copy commit back on `linkuistics-fork`. `-s`
  carries the subtree. Rebase was clean, as predicted; 380 files added, 377
  modified, 210 removed in the working copy.
- **`linkuistics-fork-next` created at `@`**, off `master@upstream`.
  `jj diff --from master@upstream --to @ --stat` shows the six `.grove/*.md`
  files and nothing else, which is the leaf's structural bar.
- `linkuistics-fork` untouched, still at `085bcf89`.
- **Toolchain pinned via the repo, not the ambient shell.** This session's PATH
  had Homebrew's cargo 1.98.0 first and no `~/.cargo/bin`, so
  `rust-toolchain.toml` (`channel = "1.96.1"`) was being ignored. Everything
  below ran with `PATH="$HOME/.cargo/bin:$PATH"` so the rustup shims honour the
  pin — cargo/rustc 1.96.1. `x86_64-pc-windows-msvc` is already installed for
  that toolchain, which `just windows-lint` needs.
- **`ZIG=/opt/homebrew/opt/zig@0.15/bin/zig`.** Ambient `zig` on PATH is 0.16.0;
  `vendor/libghostty-vt/build.zig.zon` sets `minimum_zig_version = "0.15.2"` and
  `build.rs` honours the `ZIG` env var (`build.rs:63`). Homebrew `zig@0.15` is
  installed at 0.15.2.
- **Hypothesis tested and rejected: inherited herdr session env is not the
  cause.** This session runs inside a live herdr pane (`HERDR_ENV=1`,
  `HERDR_SOCKET_PATH=~/.config/herdr/herdr.sock`, `HERDR_PANE_ID=w1K:p1`), and
  the bundled integration hooks (`src/integration/assets/*/herdr-agent-state.sh`)
  read `HERDR_SOCKET_PATH` from the environment, so a spawned test agent
  reporting to Antony's running stable server was a plausible cause. It is not:
  re-running under `env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH
  -u HERDR_ENV -u HERDR_PANE_ID -u HERDR_TAB_ID -u HERDR_WORKSPACE_ID`
  reproduced the identical four failures. The `agent target w1:p1 not found`
  message is the test's own pane naming, not the inherited `w1K:p1`.
  Clearing the vars is still correct hygiene for `cargo run` per `CLAUDE.md`,
  but it changes nothing about these tests.

### `just check` result — RED, with exactly four pre-existing test failures

Run on pristine upstream `e366a05f` with **zero fork patches applied**. Per this
leaf's charter these are recorded, **not fixed**.

Every gate except the test run is **green**:

| gate | result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo clippy --all-targets --locked -- -D warnings` | pass |
| `just windows-lint` (`x86_64-pc-windows-msvc` clippy, `-D warnings`) | pass |
| `just maintenance-test` | pass — `Ran 101 tests ... OK` (includes the vendor-patch index/reverse-apply tests) |
| `just ui-hot-path-architecture-test` | pass — `Ran 6 tests ... OK` |
| `just integration-assets-test` | pass — 5 tests, 0 fail |
| `just plugin-marketplace-test` | pass — 31 tests, 0 fail |
| `just docs-contract-test` | pass — 7 tests, 0 fail |

`cargo nextest run --locked` — **`3081 tests run: 3077 passed (1 leaky), 4 failed, 2 skipped`**

The four, verbatim:

```
FAIL herdr::live_handoff live_handoff_preserves_pane_process_io
    thread 'live_handoff_preserves_pane_process_io' panicked at tests/live_handoff.rs:905:10:
    called `Result::unwrap()` on an `Err` value: "Invalid argument (os error 22)"

FAIL herdr::live_handoff live_handoff_keeps_agent_started_pane_after_agent_exits
    thread 'live_handoff_keeps_agent_started_pane_after_agent_exits' panicked at tests/support/mod.rs:115:5:
    file did not appear at /tmp/hlh-40193-0/agent-started

FAIL herdr::live_handoff live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session
    thread 'live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session' panicked at tests/live_handoff.rs:1346:9:
    agent process was not detected: {"error":{"code":"agent_not_found","message":"agent target w1:p1 not found"},"id":"test:agent:wait-for-process"}

FAIL herdr::api_ping pane_info_and_subscriptions_expose_done_agent_status
    thread 'pane_info_and_subscriptions_expose_done_agent_status' panicked at tests/api_ping.rs:185:32:
    timed out waiting for json line
```

**Provenance of that number.** `just check`'s nextest invocation is fail-fast, so
the first two runs stopped at 88–90 of 3081 and proved nothing about the other
2993. The count above comes from a dedicated
`cargo nextest run --locked --no-fail-fast` over the whole suite. **There are no
further failures hiding behind fail-fast — the four are the complete set.**

Two candidate causes were tested and **both rejected**; the failures reproduce
identically under each control:

1. inherited herdr session env (see the running log above) — cleared, same four;
2. the harness Bash sandbox — `os error 22` on a socket read is a plausible
   sandbox denial, so the full run was repeated with the sandbox disabled. Same
   four.

All four are server-lifecycle / agent-detection integration tests that spawn real
child processes, PTYs and unix sockets. They are **not attributable to the port**,
because no port has happened yet. Whether they are broken upstream or specific to
this macOS workstation was not investigated — out of scope for this leaf.

**Warning for `hook-authority-k3`:** two of the four
(`live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session`,
`pane_info_and_subscriptions_expose_done_agent_status`) sit in the
agent-detection/hook-report area that cluster touches. That cluster's validation
starts from a red neighbourhood; diff against this list rather than expecting
green.

### Binary smoke test — pass

`env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH cargo run --locked -- --help`
builds and prints usage for herdr v0.8.2. It reports
`Config: /Users/antony/.config/herdr-dev/config.toml`, confirming the debug build
addressed its own dev namespace and not the installed stable server.
