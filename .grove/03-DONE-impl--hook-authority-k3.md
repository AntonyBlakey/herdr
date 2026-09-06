# hook-authority-k3

## Goal

Get `--source grove` pane reports working again on the new base — **by first
establishing whether the original bug still exists**, and only then porting or
reshaping the fix.

## Context

This is the one cluster whose *premise* needs re-checking, which is why it runs
while the base is still pristine.

The original fix (`vyzoqsxm`, one file, +74/−2 in `src/terminal/state.rs`):
`set_hook_authority_at` consulted `current_session_owner_conflicts` unconditionally
and cleared `persisted_agent_session` on every accepted report. So a third-party
reporter sending **no** `session_ref` — which has no session identity to conflict
with — was vetoed by, and wiped, the identity owner. Against a pane whose session
is owned by the claude integration (which only reports identity at SessionStart),
that locks the reporter out of the pane for its whole life. The fix gated both
behaviours on `session_ref.is_some()`.

Read the whole commit message; it is unusually good and states the principle:
*a hook report that makes no session-identity claim neither conflicts with, nor
clears, the identity owner.*

**Why the premise is in doubt.** Upstream has grown a lot of new machinery around
that exact site — `session_identity_only_integration()`,
`full_lifecycle_hook_authority()`, `route_full_lifecycle_hook_report()`,
`foreground_agent_confirms_hook_authority_takeover()`,
`lifecycle_hook_report_replaces_persisted_session()`. The unconditional
`current_session_owner_conflicts` call is **still there** on tip, so the fix has
almost certainly not landed — but one of those new early-return gates may
short-circuit the grove case before it ever reaches the veto, in which case
porting the old patch adds dead code, or the right fix now has a different shape.

Two tests come across with the fix and are the seam:
`session_ref_less_hook_report_is_accepted_despite_different_session_owner` and
`session_ref_less_hook_report_preserves_the_session_owner`. **They encode the
fixed behaviour, so they will pass on a port whether or not the bug is real.**
That is exactly why the reproduction comes first.

Read the patch with:

```
jj diff -r vyzoqsxm --git
```

## Done when

- The original symptom has been **reproduced or refuted** against the unpatched
  `linkuistics-fork-next` build, and which one is recorded here with the actual
  command and output. The repro from the commit message:
  ```
  herdr pane report-agent <PANE_ID> --source grove --agent grove --state working
  ```
  against a pane whose session is owned by the claude integration.
- If reproduced: the fix is ported — as the same `session_ref.is_some()` gate if
  that is still the right shape, or reshaped to fit upstream's new routing, with
  the reasoning recorded here.
- If refuted: say which upstream gate now handles it, and **do not port the
  patch**. Retire the leaf having established that; carrying a fix for a bug that
  no longer exists is a worse outcome than not porting it.
- Both tests present and passing, adapted if the fix's shape changed.
- `just check` passes — or fails only in ways `upstream-base-k2` already recorded.
- A real `herdr pane report-agent --source grove` against a claude-owned pane is
  accepted by the built binary.

## Notes

- `src/terminal/state.rs` is the **only** file this cluster touches; there is no
  entanglement with the other two leaves.
- Upstream added +851 lines to that file since the fork point (+88 since v0.8.2),
  so expect the surrounding code to be unfamiliar even though the file is the same.
- No `unwrap()` in production code; `#[allow]` needs a comment saying why.
- Commit style: lowercase conventional commit, **no AI co-author lines**.

## Decisions (running log)

### The bug is real on the new base — reproduced, not refuted

Reproduced twice on unpatched `linkuistics-fork-next` (base `e366a05f`), at the
unit seam and end to end through the CLI.

**Static read of the funnel first.** `set_hook_authority_at` now runs five
sequential vetoes. The two new gates the brief flagged are exact-match
allowlists keyed on `(source, agent_label)` — `session_identity_only_integration`
holds `herdr:hermes`/`herdr:qwen`/`herdr:antigravity_cli`, and
`full_lifecycle_hook_authority` holds `herdr:pi`/`herdr:omp`/`herdr:mastracode`/
`herdr:opencode`/`herdr:kilo`/`herdr:kimi` (`src/detect/mod.rs:316-333`).
Neither contains `("grove","grove")`, and `route_full_lifecycle_hook_report`
returns `Accept{reanchor_sequence:false}` on its first line for any
non-full-lifecycle source (`src/terminal/state.rs:871`). So the new machinery
routes *around* the grove case; it does not short-circuit it. The report still
reaches the unconditional `current_session_owner_conflicts` at
`src/terminal/state.rs:725`, and the escape hatch
`foreground_agent_confirms_hook_authority_takeover` is itself
`session_ref.as_ref().is_some_and(..)` (`state.rs:1648`), so a `None` session ref
yields `owner_conflicts && !false` → `return None`.

**Seam repro.** Both ported tests added to the unpatched tree and run:

```
$ cargo nextest run --locked -E 'test(session_ref_less_hook_report)'
Summary [0.012s] 2 tests run: 0 passed, 2 failed, 3083 skipped
  FAIL session_ref_less_hook_report_preserves_the_session_owner
    panicked: session-ref-less report should be accepted
  FAIL session_ref_less_hook_report_is_accepted_despite_different_session_owner
    panicked: a report claiming no session identity should not conflict with the session owner
```

The report is rejected outright — `mutation.is_some()` is false — which is the
veto, not the clear.

**End-to-end CLI repro**, throwaway headless session on the debug build
(`./target/debug/herdr --session repro-hookauth-… server`, isolated under
`~/.config/herdr-dev`), claude ownership established exactly as the integration
does it (`pane.report_agent_session` at SessionStart,
`src/integration/assets/claude/herdr-agent-state.sh:82`):

```
$ herdr pane report-agent-session w1:p1 --source herdr:claude --agent claude \
      --agent-session-id claude-session-abc --seq 1
$ herdr pane report-agent w1:p1 --source grove --agent grove --state working
$ herdr pane get w1:p1
  agent_status: unknown | agent_session: {herdr:claude, claude, claude-session-abc}
```

`report-agent` prints nothing whether it is accepted or vetoed, so the read above
alone is also consistent with "the command does nothing." **Positive control**,
byte-identical command against a sibling pane with no session owner:

```
$ herdr pane report-agent w1:p2 --source grove --agent grove --state working
$ herdr pane get w1:p2
  agent_status: working | agent_session: None
```

Clean-here plus dirty-there: the instrument works, and the veto is real.

### The original `session_ref.is_some()` gate is still the right shape

Both sites are unchanged upstream, so the patch applies as written. The one new
neighbour worth checking is `reanchor_sequence`, added next to the clear site as
`if session_ref.is_some() || reanchor_sequence`. Gating the clear on
`session_ref.is_some()` alone cannot diverge from it: **both** paths that set
`reanchor_sequence = true` require a `Some` session ref — the HookClear branch
matches `(Some(previous), Some(incoming))` (`state.rs:912-915`), and
`full_lifecycle_hook_report_has_fresh_session_after_stale_session` is
`session_ref.as_ref().is_some_and(..)` (`state.rs:1028`). So
`session_ref.is_some() || reanchor_sequence` ≡ `session_ref.is_some()` and no
reshaping is needed.

### Suite after the fix: the four baseline failures and nothing else

```
$ cargo nextest run --locked --no-fail-fast
Summary [17.054s] 3083 tests run: 3079 passed, 4 failed, 2 skipped
  FAIL herdr::live_handoff live_handoff_preserves_pane_process_io
  FAIL herdr::live_handoff live_handoff_keeps_agent_started_pane_after_agent_exits
  FAIL herdr::live_handoff live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session
  FAIL herdr::api_ping  pane_info_and_subscriptions_expose_done_agent_status
```

Item-by-item identical to the four `upstream-base-k2` recorded, and the totals
move by exactly the two tests this leaf adds: baseline `3077 passed / 4 failed /
2 skipped` of 3081 → `3079 / 4 / 2` of 3083. No port regression.

The production diff is +74/−2 in `src/terminal/state.rs` — the same footprint as
the fork's original `vyzoqsxm`. **Two things made the re-home non-trivial even so:**

- The patched line is **no longer unique in the file**. Upstream grew a second
  function (the `pane.report_agent_session` identity path, `state.rs:1546`) with a
  byte-identical `let owner_conflicts = self.current_session_owner_conflicts(…)`.
  The disambiguator is the following call — `foreground_agent_confirms_hook_authority_takeover`
  (lifecycle, `Option<AgentSessionRef>`) vs `foreground_agent_confirms_different_owner_takeover`
  (identity, non-optional session ref, where the new gate would not typecheck).
- `session_ref` is **shadowed between the two hunks** by the
  `session_ref.map(…)` rebinding upstream added at `state.rs:677`. `Option::map`
  preserves `is_some`, so both hunks still test the same predicate.

### `just check`: clean apart from the four recorded baseline failures

`just check` (= `ci` → `lint`, nextest, script gates; then `windows-lint`,
`docs-contract-test`) exits non-zero, but only at its fail-fast nextest step and
only on the four `upstream-base-k2` failures. `lint` runs *before* nextest and
passed. Everything the fail-fast abort skipped was then run explicitly under
`set -e`:

```
=== lint (fmt + clippy) ===            ok
=== windows-lint ===                   ok  (x86_64-pc-windows-msvc, -D warnings)
=== maintenance-test ===               ok  (incl. vendor patch index/reverse-apply)
=== ui-hot-path-architecture-test ===  ok
=== integration-assets-test ===        ok
=== plugin-marketplace-test ===        ok
=== docs-contract-test ===             ok
=== ALL NON-NEXTEST GATES PASSED ===
```

### Live confirmation on the built binary

Rebuilt after `touch src/terminal/state.rs` — a bare `cargo build` reported
`Finished in 0.11s`, which reads the same whether the binary was already current
or cargo missed the edit, so the forced recompile is what makes the run below
evidence. Fresh throwaway headless session, same four commands as the repro:

```
  w1:p1  agent_status: working  | agent_session: {herdr:claude, claude, claude-session-abc}
  w1:p2  agent_status: working  | agent_session: None            (unowned control)
```

`w1:p1` was `unknown` before the fix and is `working` now — **accepted** — while
`agent_session` is still claude's — **not cleared**. Both halves of the principle,
on the real binary.

**Negative control — the fix is not over-broad.** A report that *does* make an
identity claim must still be vetoed. Using an official source, since a
session-ref-carrying grove report is impossible (below):

```
  w1:p4  agent_status: unknown  | agent_session: {herdr:claude, claude, claude-session-neg}
      # herdr:codex/codex --state working --agent-session-id codex-session-1
      # against a claude-owned pane: still vetoed, owner preserved
```

**Scope note found while building that control.** `pane report-agent
--source grove --agent grove --agent-session-id <id>` silently *discards* the id:
`session_ref_from_report` (`src/agent_resume.rs:53-61`) returns `None` unless
`is_official_agent_source(source, agent)`, and `("grove","grove")` is not one. So
every grove report is session-ref-less by construction and always takes the new
gate. Nothing to change — but it means the grove path cannot be exercised with a
session ref, which is why the negative control above uses `herdr:codex`.

Throwaway session stopped and deleted; `session list` shows only the pre-existing
stopped `default`. No parent-session pane was created or closed.

### `linkuistics-fork-next` advanced to this commit

The brief has Antony **rename** `linkuistics-fork-next` to `linkuistics-fork`
after the grove ends, and a jj bookmark rename keeps its position. `upstream-base-k2`
created the bookmark at the base commit and recorded no per-leaf discipline, so
leaving it there would hand Antony a branch containing the base and **none** of
the three cluster ports. Advanced it with `jj bookmark set linkuistics-fork-next -r @`
as part of this commit; later leaves should do the same. Local only — nothing here
pushes to `origin`, and `linkuistics-fork` is untouched.
