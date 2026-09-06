# plan-k1

## Goal

Establish what "update herdr fork to latest upstream" means concretely, and either
do it in this session or grow a tree that can.

## Context

Repository facts established at bootstrap (2026-09-06):

- This is a **secondary jj workspace** `update-to-latest-upstream`; the default
  workspace is `../herdr`. Remotes: `origin` = `AntonyBlakey/herdr` (Antony's
  fork), `upstream` = `ogulcancelik/herdr` (canonical).
- The fork's work lives on bookmark **`linkuistics-fork`**
  (`085bcf89 docs: document inactive pane haze`).
- Fetched `upstream` at bootstrap. `master@upstream` is now
  `9e9bc8a1 feat: manage multiple ssh machines from one client (#3670)`.
- **Divergence: 281 upstream commits** the fork does not have; 925 files,
  ~151k insertions / ~94k deletions since the fork point
  (`73d92004 ci: remove approved contributor bypass`, ≈ v0.7.5). Upstream is now
  at tag v0.8.2.
- The fork's own footprint is small by comparison: **21 files, ~1873 insertions,
  54 deletions**, in three feature clusters:
  1. **`ui.layout` API + `herdr ui layout` CLI verb** — sidebar/tab-bar/pane
     geometry (`src/api/schema/ui.rs`, `src/app/api/ui.rs`, `src/cli/ui.rs`).
  2. **Hook-authority fix** (`src/terminal/state.rs`) — gate owner-conflict veto
     and persisted-session clearing on `session_ref.is_some()`, so a reporter
     that makes no session-identity claim (e.g. `herdr pane report-agent
     --source grove`) is not locked out of a pane.
  3. **Pane border colours + inactive-pane haze** (`src/config/model.rs`,
     `src/ui/haze.rs`, `src/ui/panes.rs`, plus docs/config-reference).

- **The upstream substrate under those patches has been rewritten**, which is the
  real cost here, not the commit count:
  - `src/app/mod.rs` −3964 lines, `src/app/state.rs` −1740, `src/ui.rs` −1448
    (module splits).
  - `src/ui/panes.rs` ±609, `src/terminal/state.rs` +851, `src/app/api.rs` ±767,
    `src/api/server.rs` +272, `src/config/model.rs` +300, `src/cli.rs` +193.
  - Only the fork's four *new* files (`src/api/schema/ui.rs`, `src/app/api/ui.rs`,
    `src/cli/ui.rs`, `src/ui/haze.rs`) are untouched upstream — because they do
    not exist there.

- **Overlap check against upstream:**
  - `ui.layout`: still absent upstream. Upstream added `src/app/api/layouts.rs`
    (saved pane layouts) and a `plugins/` API subsystem — adjacent, not the same
    thing.
  - Border colours / haze: upstream landed `feat: make outer pane borders
    configurable (#2535)` and now has `pane_borders`, `pane_outer_borders`,
    `show_agent_labels_on_pane_borders` and a theme accent colour — but **no**
    per-state border fg/bg keys and **no** haze. Partial overlap in the same
    config namespace.
  - Hook-authority fix: **not** applied upstream — `set_hook_authority_at` still
    consults `current_session_owner_conflicts` unconditionally. But the function
    has grown substantially around it (`session_identity_only_integration`,
    `full_lifecycle_hook_authority`, `route_full_lifecycle_hook_report`,
    `foreground_agent_confirms_hook_authority_takeover`), so whether the original
    symptom still reproduces is an open empirical question.

- Per `CLAUDE.md`, the acting account is an **external contributor** to
  `ogulcancelik/herdr`, not a maintainer.

- **All three clusters have live consumers outside this repo** (looked up, not
  assumed):
  - `ui.layout` — **modaliser** calls `(herdr-query "ui.layout" '())` and depends
    on the response envelope (canvas, and `workspaces`/`agents`/`tabs` entries
    with rects) for chip painting:
    `~/.config/modaliser/sys/scheme/lib/modaliser/muxes/herdr.sld`,
    `blocks/herdr-list.sld`, `util.sld`.
  - Border colours + haze — `~/.config/herdr/config.toml` sets
    `inactive_pane_border_bg`, `inactive_pane_haze`, `inactive_pane_haze_color`
    under `[ui]`. None of the six fork keys exist upstream.
    (`active_pane_border_fg` is present but commented out.)
  - Hook-authority fix — `--source grove` reports are how grove drives herdr
    panes.
  So "which clusters survive" is not an open question: all three are in daily use.
  The user's other `[ui]` keys (`agent_panel_sort`, `[ui.sidebar.spaces]`,
  `show_agent_labels_on_pane_borders`) are upstream features, not fork ones.

## Done when

- The human's intent for this workstream is recorded in their own words.
- The root `BRIEF.md` states the goal and the done-criteria rollup.
- Either the work is complete in this session, or a tree exists that can complete
  it.

## Notes

`jj git fetch --remote upstream` was run during bootstrap. No bookmarks were
moved and no working-copy content outside `.grove/` was touched.

## Decisions (running log)

**Fork purpose — personal fork, kept current.** `linkuistics-fork` is a build
Antony runs, not a staging area for upstream PRs. The deliverable is a working
fork on latest upstream in whatever shape is cheapest to maintain; the three
clusters need not stay independently cherry-pickable. Evidence weighed:
`CLAUDE.md`'s external-contributor guardrail requires GitHub Discussions and
maintainer approval before the two feature clusters could be PR'd upstream at
all, so preserving separability would buy an option that cannot be exercised
cheaply anyway.

**Integration strategy — port forward, one cluster at a time.** New base is
upstream master; the current fork diff is treated as a *specification*, and each
cluster is re-applied onto the new module layout as fresh commits rather than
merged or rebased. Rationale: the fork diff is overwhelmingly additive (+1873/−54),
which ports by hand cleanly; whereas upstream split every host file the patches
sit in (`src/app/mod.rs` −3964, `src/app/state.rs` −1740, `src/ui.rs` −1448), so
a merge or rebase would raise relocation conflicts — "find where this code went" —
resolved under conflict markers with the original context already destroyed. A
rebase would additionally re-raise them once per fork commit, twelve times,
through two stale merge commits.

**Base ref — upstream master tip** (`9e9bc8a1`), not tag `v0.8.2`. Evidence: the
module splits landed *after* v0.8.2, in the last two weeks (99 commits, 288 source
files; `src/app/mod.rs` −4325 and `src/ui.rs` −1519 post-tag). Basing on the tag
would defer rather than avoid the re-homing, and would pay it twice — once onto
the monolith, once onto the split. The fork's `src/ui/haze.rs` is already a
module, so it fits the post-split layout better than the one it was written
against.

**`ui.layout` wire shape is preserved verbatim.** Not grilled — the evidence
leaves no trade-off: `PaneLayoutRect` and `PaneLayoutPane` are byte-identical
between the fork and upstream tip, so the schema types in `src/api/schema/ui.rs`
port unchanged and modaliser needs no edit. Only the *implementation*
(`src/app/api/ui.rs`, 553 lines, reads `AppState` geometry) must be re-homed
against the gutted `src/app/state.rs` / `src/app/mod.rs`. Extending `ui.layout`
to cover upstream's newer UI surfaces (plugin panes, the `spaces` sidebar) is
**out of scope** — horizon work, not this sync.

**Verification bar — `just check` + a real run, symptom-first.** Per cluster:
`just check` must pass; the built binary must be run and the cluster's
externally-observable effect confirmed (haze and border colours render; modaliser's
`ui.layout` query still paints chips); and the hook-authority cluster must
**first reproduce its original symptom against an unpatched upstream build**
before any fix is ported. Rationale: the fork's 27 tests port with the code and
encode the *fixed* behaviour, so they pass whether or not the bug still exists —
and upstream grew `session_identity_only_integration()` and
`full_lifecycle_hook_authority()` around the patched site, either of which may
have made the fork's gate redundant or may need a differently-shaped fix.

**Test seams agreed with the human** (they already exist in the fork and port
with the code):
- `ui.layout` — `src/app/api/ui.rs` geometry tests (13), `src/api/schema/tests.rs`
  schema round-trip (1), and the golden `docs/next/api/herdr-api.schema.json`.
- hook authority — `TerminalState::set_hook_authority_at`, via
  `session_ref_less_hook_report_is_accepted_despite_different_session_owner` and
  `session_ref_less_hook_report_preserves_the_session_owner` (2).
- borders + haze — `src/ui/haze.rs` colour math (11) and config parsing in
  `src/config/model.rs`.

**Landing — a new bookmark; `linkuistics-fork` stays put; nothing is pushed.**
The ported work is built on a fresh bookmark off `master@upstream`.
`linkuistics-fork` keeps pointing at the pre-port head, for two reasons: it *is*
the port's specification (every leaf reads
`jj diff --from 'fork_point(linkuistics-fork | master@upstream)' --to linkuistics-fork`),
and it stays a runnable fallback. Port-forward shares no commits with it, so
moving it would be a non-fast-forward move needing a force-push. **No session in
this grove pushes to `origin`** — Antony moves the bookmark and pushes himself,
once the new build has survived daily use.

**Decomposition — this session cuts the leaves; no separate `planning` leaf.**
The decomposition is settled and evidence-based (per-cluster file sets computed
below), so a `planning` session would only re-derive it. Cluster independence was
verified by file set, not assumed:
- **A — `ui.layout`** (`qympmurl`, `nqvvrqtk`): `src/api/schema/{ui,response,tests}.rs`,
  `src/api/schema.rs`, `src/api/server.rs`, `src/app/api/ui.rs`, `src/app/api.rs`,
  `src/cli/{ui,spec}.rs`, `src/cli.rs`, `src/main.rs`, `docs/next/api/herdr-api.schema.json`,
  plus 1 line in `src/app/mod.rs`, 4 in `src/app/state.rs`, 2 in `src/ui.rs`.
- **B — hook authority** (`vyzoqsxm`): `src/terminal/state.rs` **only**.
- **C — pane colours + haze** (`tpxyyoyo`, `pzsyyllly`, `trsvxtxu`, `opoxotxw`,
  `kznxtusu`, `osrwoxtq`, `xltnzmqp`): `src/config/model.rs`, `src/ui/panes.rs`,
  `src/ui/haze.rs`, `src/ui.rs`, `src/app/mod.rs` (52), `src/app/state.rs` (77),
  and the two docs config-reference files.

Borders and haze are **one** cluster, not two: they share `src/ui/panes.rs`,
`src/config/model.rs`, `src/app/state.rs` and the config-reference JSON, and the
`refactor: move color primitives into a haze module` commit refactors the border
code the earlier commits introduced. A and C overlap only trivially
(`src/app/mod.rs`, `src/app/state.rs`, `src/ui.rs` — 7 lines total on A's side),
which ordering handles.

**Absence claims verified, not assumed.** Two claims in the root brief are
repo-wide negatives, so they were checked against the whole upstream tree
(`git --git-dir=/Users/antony/Development/herdr/.git grep -l <pat> 9e9bc8a1`)
with both controls:

- *Positive control*: `pane_outer_borders` found in `src/app/agent_resume.rs`,
  `src/app/api/panes.rs`, `src/app/mod.rs`; the same command with the pattern
  mutated to `pane_outer_borderZZ` returned nothing — so the instrument can read
  empty.
- *Cross-tree control*: all six config keys and the `ui.layout` pattern are found
  in the fork tree by the identical command.
- *Result*: all six keys, `ui\.layout|UiLayout|ui_layout`, and the whole word
  `haze` return **0 files across the entire upstream tree**.
- One anomaly chased: the fork match list included `src/app/input/navigate.rs`,
  which is not in cluster A's file set. It is a substring false positive —
  `"tui.layout.set_split_ratio"` matched `ui\.layout`. Unrelated to the fork;
  upstream has since changed that line.

`PaneLayoutRect` / `PaneLayoutPane` were likewise confirmed byte-identical
between the two commits by extracting and diffing both. The first attempt at
that comparison reported "identical" from **two empty extractions** — caught only
because the control (`UiConfig`, known to differ) also reported identical. The
working comparison extracts 11 lines a side and the control differs 65 vs 68.

