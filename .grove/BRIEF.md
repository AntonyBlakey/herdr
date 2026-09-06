# herdr.update-to-latest-upstream — brief

## Goal

Get Antony's herdr fork — three feature clusters he uses every day — running on
current upstream `master`, by **porting each cluster forward** onto the new
module layout rather than merging or rebasing.

`linkuistics-fork` had drifted 281 commits behind `ogulcancelik/herdr`
(≈ v0.7.5 → past v0.8.2). Its own footprint is small and almost entirely
additive (+1873/−54 across 21 files), but upstream split every host file those
patches live in — `src/app/mod.rs` −3964, `src/app/state.rs` −1740,
`src/ui.rs` −1448 — so the patches' targets have *moved*, not merely changed.
That is why this is a port and not a merge.

## Done when

All three clusters run on a build based on upstream `master` tip, and for each:

- `just check` passes (formatting + `cargo nextest` + maintenance script tests);
- the built binary has been **run**, and the cluster's externally-observable
  effect confirmed by hand — not inferred from green tests;
- the cluster's own tests came across with it and pass (see **Test seams**).

Then Antony moves `linkuistics-fork` and pushes. **No session here pushes to
`origin`.**

## Decomposition

Positions encode dependency, then risk.

1. **`upstream-base`** — the new bookmark off `master@upstream`, proven green
   before any patch lands. Everything downstream needs it, and it is what makes a
   later failure attributable: if `just check` is red *here*, it is upstream's,
   not the port's. It is also the **unpatched build** the next leaf must
   reproduce against.
2. **`hook-authority`** — smallest (one file), most load-bearing (grove drives
   herdr panes through it), and the only one whose *premise* needs re-checking.
   It must run while the base is still pristine.
3. **`ui-layout`** — highest re-homing risk: 553 lines of `src/app/api/ui.rs`
   read `AppState` geometry, and `state.rs`/`mod.rs` are the most heavily gutted
   files upstream. Front-loaded so its cost is known early. modaliser is dark
   until it lands.
4. **`pane-colors`** — most self-contained; `src/ui/haze.rs` is already a module
   and fits the post-split layout better than the monolith it was written for.

3 and 4 are independent of each other and could swap.

## Pointers

- **The port's specification is the old branch.** Every leaf reads it as:
  `jj diff --from 'fork_point(linkuistics-fork | master@upstream)' --to linkuistics-fork --git -- <paths>`
  Do not move or delete `linkuistics-fork`.
- **The new bookmark is `linkuistics-fork-next`**, created off `master@upstream`
  by `upstream-base-k2`. Every leaf here builds on it. It is renamed to
  `linkuistics-fork` by Antony, by hand, after the grove ends.
- **This grove's own `.grove/` history starts on the old line.** The
  `requirements` commit is a child of `linkuistics-fork`'s head, because that is
  where the working copy was when the driver scaffolded the tree.
  `upstream-base-k2` rebases it onto `master@upstream` as its first act — see
  that leaf.
- Project rules: `CLAUDE.md` — validate with `just check`, no `unwrap()` in
  production code, `#[allow]` needs a reason, platform code compile-gated under
  `src/platform/`, lowercase conventional commits with **no AI co-author lines**.
- Antony is an **external contributor** to `ogulcancelik/herdr`, not a
  maintainer. Nothing here opens issues or PRs upstream.
- Vendored `libghostty-vt` needs **Zig 0.15.2**; upstream also moved
  `vendor/portable-pty.patches.md`. `just check` runs maintenance tests that
  verify local vendor patches are indexed and reverse-apply — the fork adds no
  vendor patches of its own, so any failure there is upstream's.
- Unreleased docs live in `docs/next/`; the fork's doc changes are already there
  (`docs/next/api/herdr-api.schema.json`,
  `docs/next/website/src/content/docs/configuration.mdx`,
  `docs/next/website/src/data/config-reference.json`). Do **not** touch root
  `README.md`, root `CHANGELOG.md`, or `website/src/content/docs/`.

## Test seams

Agreed with Antony. All three sets already exist in `linkuistics-fork` and port
with the code — porting a cluster without its tests is not porting it.

| cluster | seam | tests |
|---|---|---|
| `ui.layout` | `src/app/api/ui.rs` geometry; `src/api/schema/tests.rs` round-trip; golden `docs/next/api/herdr-api.schema.json` | 13 + 1 |
| hook authority | `TerminalState::set_hook_authority_at` | 2 |
| pane colours + haze | `src/ui/haze.rs` colour math; config parsing in `src/config/model.rs` | 11 |

The seams do **not** cover: that haze and borders actually render, that
modaliser's `ui.layout` query still paints chips, or whether the hook fix is
still fixing anything. Hence the run-it-for-real bar in **Done when**.

## Notes

**Live consumers outside this repo** — this is why all three clusters carry
forward and none is dropped:

- **modaliser** calls `(herdr-query "ui.layout" '())` and depends on the response
  envelope — canvas, and `workspaces`/`agents`/`tabs` entries with rects — in
  `~/.config/modaliser/sys/scheme/lib/modaliser/muxes/herdr.sld`,
  `blocks/herdr-list.sld`, `util.sld`. `PaneLayoutRect` and `PaneLayoutPane` are
  byte-identical between the fork and upstream tip, so **the wire shape is
  preserved verbatim and modaliser is not edited**.
- `~/.config/herdr/config.toml` sets `inactive_pane_border_bg`,
  `inactive_pane_haze` and `inactive_pane_haze_color`. None of the six fork keys
  exist upstream. (`agent_panel_sort`, `[ui.sidebar.spaces]` and
  `show_agent_labels_on_pane_borders` in that file *are* upstream features.)
- grove reports pane state with `herdr pane report-agent --source grove`, which
  is exactly what the hook-authority fix unblocks.

**Upstream overlap, checked.** `ui.layout` is still absent upstream
(`src/app/api/layouts.rs` is saved pane layouts, a different thing). Upstream
landed `feat: make outer pane borders configurable (#2535)` and now has
`pane_borders`, `pane_outer_borders` and a theme accent colour — but no per-state
border fg/bg keys and no haze, so cluster C is an addition in a shared namespace,
not a duplicate.

## On the horizon

- Extending `ui.layout` to cover upstream's newer UI surfaces — plugin panes, the
  `spaces` sidebar. Explicitly out of scope for the sync.
- Whether any cluster is worth proposing upstream. The two feature clusters would
  need GitHub Discussions and maintainer approval first; only the hook fix fits
  herdr's automated `fix:` intake budget (1 file, 76 lines). Not this grove's
  business.
