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
