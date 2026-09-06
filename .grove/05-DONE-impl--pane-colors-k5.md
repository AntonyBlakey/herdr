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

### The theme system is not the cleaner route; the fork's `[ui]` keys stand

The task asked whether upstream's theme system now reaches this more cleanly.
Read: `src/ui/panes.rs` resolves a border colour as
`if focused { app.palette.accent } else { app.palette.overlay0 }` at both border
sites (cells, `src/ui/panes.rs:488`; titles, `src/ui/panes.rs:656`) — byte-identical
to the fork point. The theme surface that could override those is
`Palette::with_overrides` / `with_mode_overrides` (`src/app/state.rs:560-672`),
driven by `[theme.colors]` / `[theme.dark]` / `[theme.light]`.

That route is rejected: `accent` and `overlay0` are used across the whole UI
(sidebar, tabs, dialogs), so overriding them to restyle pane borders restyles
everything else too — the opposite of what the docs promise ("without restyling
the rest of the UI"). It also has no background component and no notion of a
blend ratio, so `*_border_bg`, `inactive_pane_haze` and `inactive_pane_haze_color`
have nowhere to live there. Dedicated `[ui]` keys with the palette as fallback
stay correct, and `AppState::pane_border_style` remains the single resolution
point both border sites call.

### Upstream deleted the inactive-pane dim, so one fork test cannot port

`refactor: render the shell in the client (#3487)` removed the
`should_dim = !info.is_focused && multi_pane && !terminal_active` pass from
`render_panes` — the server no longer knows the client's mode, so it can no
longer dim on it (verified: no `Modifier::DIM` remains anywhere under the pane
surface path). The fork's haze was written as a *replacement* for that dim and
kept it as the `else` arm; here there is no `else` arm to keep.

Consequence: the fork test `haze_disabled_keeps_navigate_mode_dim` guards
behaviour that no longer exists upstream and is dropped rather than ported. Its
other half — that haze off leaves the cell background untouched — is folded into
a new test so the "off means off" assertion is not lost. The remaining ten
`src/ui/haze.rs` tests and every border/haze test in `panes.rs` port as-is.

### Where the six values land, and the one hoist away from the fork

Plumbing is the fork's shape — the host files moved but the anchor lines did not:

| what | where |
|---|---|
| six `Option<String>`/`f32` config fields, `UiConfig::inactive_pane_haze()` | `src/config/model.rs` |
| six resolved `Option<Color>`/`f32` fields, `AppState::pane_border_style` | `src/app/state.rs` |
| config → state, at construction and on reload | `src/app/mod.rs` |
| colour primitives, `HazeContext`, `resolve`, `haze_style` | `src/ui/haze.rs` (new) |
| `haze_context`, the haze pass, both border-style call sites | `src/ui/panes.rs` |

One deliberate departure from the fork: **`haze_context(app)` is resolved once per
frame, not once per pane.** The fork called it inside the per-pane loop. `CLAUDE.md`
classifies render work as multiplicative (per cell × panes × frames), and the
context depends only on config and the host theme, so it is hoisted above the loop
and gated on `pane_infos.len() > 1` — a single-pane workspace now pays one boolean
instead of a context build. The haze write itself is unavoidably per-cell, but only
for unfocused panes when the ratio is above zero, which is off by default.

`type Rgb`, `mix_rgb` and `color_to_rgb` moved out of `panes.rs` into `haze.rs`
exactly as `kznxtusu` did, and `panes.rs` imports them back — `color_to_rgb` gains
the xterm-256 `Indexed` arm the old one returned `None` for, which is what makes
hazing an indexed-colour cell work.

### Verified against Antony's unmodified config, on screen

`Done when` asks for the built binary run against `~/.config/herdr/config.toml`
unmodified. Two traps, both hit and cleared:

- A **debug** build reads `~/.config/herdr-dev/`, not `~/.config/herdr/`
  (`src/config/io.rs:24`). A first run "against Antony's config" was silently
  reading the dev config — dark palette, no keys — and read as *the feature does
  not work*. The real run sets
  `HERDR_CONFIG_PATH=$HOME/.config/herdr/config.toml`, which points at that file
  unmodified rather than editing it.
- `config check` returning `ok` proves the keys are recognised only because the
  same command was watched to fail: an added `bogus_key_control` reports
  `unknown config key ui.bogus_key_control`, and `inactive_pane_haze =
  "not-a-number"` reports a typed parse error. Both controls came back dirty
  before the clean read was credited.

Evidence: a PTY harness runs the debug binary at 120x40, splits `w1:p1` right via
`herdr pane split`, and captures the raw redraw. With Antony's config
(`one-light`, `inactive_pane_border_bg = "#dddddd"`, `inactive_pane_haze = 0.1`,
`inactive_pane_haze_color = "#000000"`) the SGR runs are:

- active pane border `38;2;64;120;242` / bg default — one-light accent, and no
  background because `active_pane_border_*` are commented out in his config;
- inactive pane border `38;2;160;161;167;48;2;221;221;221` — one-light `overlay0`
  over **#dddddd**;
- inactive pane cells `48;2;225;225;225` — one-light `panel_bg` #fafafa blended
  10% toward black (250 × 0.9 = 225), against the focused pane's untouched
  default background;
- inactive pane text `38;5;6` → `38;2;0;120;120`, `38;5;1` → `38;2;120;0;0`,
  `38;5;4` → `38;2;0;0;120` — each host palette colour blended at
  0.1 × `HAZE_FG_FACTOR` = 0.06 (128 × 0.94 = 120), so text recedes less than its
  background, exactly as designed.

The two keys his config leaves commented out were exercised separately with a
copy of it setting `active_pane_border_fg = "#ff0000"`,
`inactive_pane_border_fg = "#00ff00"`, `inactive_pane_haze = 0.5`,
`inactive_pane_haze_color = "#ff00ff"`; the capture shows `38;2;255;0;0`,
`38;2;0;255;0;48;2;221;221;221` and hazed cells at `48;2;253;125;253`
(= mix(#fafafa, #ff00ff, 0.5)). All six keys reach the screen.

### `just check` — the four baseline failures and nothing else

`cargo nextest run --locked --no-fail-fast`: **3124 tests run, 3120 passed,
4 failed, 2 skipped.** The four are item-for-item the ones
`upstream-base-k2` recorded on pristine upstream:
`live_handoff_preserves_pane_process_io`,
`live_handoff_keeps_agent_started_pane_after_agent_exits`,
`live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session`,
`pane_info_and_subscriptions_expose_done_agent_status`.

Every other gate is green run individually past `ci`'s fail-fast: `just lint`
(fmt + clippy), `just maintenance-test`, `just ui-hot-path-architecture-test`,
`just integration-assets-test`, `just plugin-marketplace-test`,
`just windows-lint`, `just docs-contract-test`. `scripts/config_reference_check.py`
also passes, which is what ties the six new `[ui]` keys to their
`config-reference.json` rows.
