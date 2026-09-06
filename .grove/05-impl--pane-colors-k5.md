# pane-colors-k5

## Goal

Re-home configurable pane border colours and inactive-pane haze onto the new
module layout, so that Antony's existing `~/.config/herdr/config.toml` works
unchanged.

## Context

The most self-contained of the three ports, and the one whose absence is visible
every day. Seven commits, ~700 added lines, and `src/ui/haze.rs` is already a
standalone module — which fits the post-split upstream layout *better* than the
`src/ui.rs` monolith it was originally written against.

Specification:

```
jj diff --from 'fork_point(linkuistics-fork | master@upstream)' --to linkuistics-fork --git \
  -- src/config/model.rs src/ui/haze.rs src/ui/panes.rs src/ui.rs \
     src/app/mod.rs src/app/state.rs \
     docs/next/website/src/data/config-reference.json \
     docs/next/website/src/content/docs/configuration.mdx
```

Commits, in order — **borders and haze are one cluster, not two**: `kznxtusu`
refactors the border code the earlier commits introduced, moving colour
primitives into the haze module.

| commit | what |
|---|---|
| `tpxyyoyo` | border colour config keys |
| `pzsyyllly` | apply configurable colours to pane borders |
| `trsvxtxu` | docs |
| `opoxotxw` | haze config keys |
| `kznxtusu` | **refactor**: colour primitives out of `panes.rs` into `haze.rs` |
| `osrwoxtq` | haze inactive panes toward a configurable colour |
| `xltnzmqp` | docs |

The six config keys, none of which exist upstream:
`active_pane_border_fg`, `active_pane_border_bg`, `inactive_pane_border_fg`,
`inactive_pane_border_bg`, `inactive_pane_haze`, `inactive_pane_haze_color`.

**Upstream overlaps this namespace without duplicating it.** `feat: make outer
pane borders configurable (#2535)` landed, and tip now has `pane_borders`,
`pane_outer_borders`, `show_agent_labels_on_pane_borders` and a theme accent
colour. So the `[ui]` config struct and the border-drawing path in
`src/ui/panes.rs` (±609 since the fork point, ±456 since v0.8.2) have both moved
under you. Read how upstream now resolves a border colour before adding keys
beside it — the fork's approach may now be reachable more cleanly through the
theme system, and if so, take the cleaner route and record why. The **keys' names
and semantics** are what must not change, not their implementation.

## Done when

- All six keys parse and take effect. The acceptance test is
  Antony's live config, which sets `inactive_pane_border_bg = "#dddddd"`,
  `inactive_pane_haze = 0.1` and `inactive_pane_haze_color = "#000000"` — run the
  built binary against `~/.config/herdr/config.toml` **unmodified** and look at
  it. Haze at 0.1 is subtle; compare an active and an inactive pane side by side,
  and try a larger value if you need to convince yourself it is wired.
- The 11 colour-math tests in `src/ui/haze.rs` and the config-parsing tests in
  `src/config/model.rs` come across and pass.
- `docs/next/website/src/data/config-reference.json` and
  `docs/next/website/src/content/docs/configuration.mdx` carry the six keys.
- `just check` passes — or fails only in ways `upstream-base-k2` already recorded.

## Notes

- Docs go in `docs/next/` only. Do **not** touch root `README.md`, root
  `CHANGELOG.md`, or `website/src/content/docs/` — `CLAUDE.md` is explicit.
- `agent_panel_sort`, `[ui.sidebar.spaces]` and
  `show_agent_labels_on_pane_borders` in Antony's config are **upstream**
  features, not fork ones. If they misbehave, that is not this leaf's problem.
- `src/app/mod.rs` (52 lines) and `src/app/state.rs` (77) are shared with
  `ui-layout-k4`, which will already have landed its 7 lines there.
- Colour parsing: check whether upstream now has a shared colour-string parser
  worth using instead of the fork's own.
- No `unwrap()` in production code. Lowercase conventional commit, no AI
  co-author lines.

## Decisions (running log)
