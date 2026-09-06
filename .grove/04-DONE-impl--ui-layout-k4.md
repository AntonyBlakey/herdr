# ui-layout-k4

## Goal

Re-home the `ui.layout` API method and the `herdr ui layout` CLI verb onto the
new module layout, **preserving the wire shape byte-for-byte** so modaliser needs
no edit.

## Context

The largest and riskiest port: ~950 added lines, of which `src/app/api/ui.rs`
(553) reads `AppState` geometry — and `src/app/state.rs` (−1740) and
`src/app/mod.rs` (−3964) are the two most heavily gutted files upstream. The code
you are porting has not merely changed around it; **its host modules were split
out from under it**, so expect to spend the session finding where sidebar, tab-bar
and pane geometry now live, not resolving conflicts.

The specification is the old branch:

```
jj diff --from 'fork_point(linkuistics-fork | master@upstream)' --to linkuistics-fork --git \
  -- src/api/schema/ui.rs src/api/schema/response.rs src/api/schema/tests.rs \
     src/api/schema.rs src/api/server.rs src/app/api/ui.rs src/app/api.rs \
     src/app/mod.rs src/app/state.rs src/ui.rs src/cli/ui.rs src/cli/spec.rs \
     src/cli.rs src/main.rs docs/next/api/herdr-api.schema.json
```

Two commits: `qympmurl` (API method, 11 files) and `nqvvrqtk` (CLI verb, 4 files).

**The wire shape is fixed, and that constraint is cheap to hold.** modaliser
calls `(herdr-query "ui.layout" '())` from
`~/.config/modaliser/sys/scheme/lib/modaliser/muxes/herdr.sld`,
`blocks/herdr-list.sld` and `util.sld`, and depends on the envelope: a canvas
(width, height) plus `workspaces` / `agents` / `tabs` entries carrying rects.
`PaneLayoutRect` and `PaneLayoutPane` in `src/api/schema/panes.rs` are
**byte-identical** between the fork and upstream tip, so `src/api/schema/ui.rs`
should port essentially unchanged. If you find yourself needing to change the
response shape, that is a signal to stop and reconsider the implementation, not
to change the contract — modaliser is out of this grove's scope.

Per `CLAUDE.md`'s runtime/client boundary: pane/agent metadata and terminal state
are server/runtime concerns and belong on the JSON API path; sidebar layout,
colours and selection are TUI presentation. `ui.layout` deliberately straddles
this — it exposes *geometry* of TUI surfaces over the API. It already exists in
that shape and this leaf is a port, not a redesign; do not relitigate it, but do
not extend it either.

## Done when

- `ui.layout` returns the same JSON envelope as on `linkuistics-fork`. Diff a
  live response against one from the old build if you can get both running;
  otherwise diff `docs/next/api/herdr-api.schema.json` between the two branches
  and account for every difference.
- `herdr ui layout` works from the CLI.
- The 13 tests in `src/app/api/ui.rs` and the schema round-trip test in
  `src/api/schema/tests.rs` come across and pass.
- `just check` passes — or fails only in ways `upstream-base-k2` already recorded.
- **modaliser paints chips against the built binary.** This is the real
  acceptance test and the tests cannot see it.

## Notes

- `src/app/api/layouts.rs` upstream is *saved pane layouts*, a different feature.
  Do not merge into it or take its naming.
- Upstream added a `plugins/` API subsystem and `src/app/api/responses.rs` /
  `api_helpers.rs`; the registration idiom for a new API method has probably
  changed. Read how a neighbouring method registers itself on tip rather than
  reproducing the fork's call sites from memory.
- `src/app/mod.rs`, `src/app/state.rs` and `src/ui.rs` are shared with
  `pane-colors-k5`, but this cluster only adds 7 lines across the three of them.
- Extending `ui.layout` to cover plugin panes or the `spaces` sidebar is
  **explicitly out of scope** (root brief, *On the horizon*).
- No `unwrap()` in production code. Lowercase conventional commit, no AI
  co-author lines.

## Decisions (running log)

### The port is a re-architecture, not a re-homing — geometry crossed a process boundary

The task's premise was that `ui.layout`'s inputs moved *module*. They moved
*process*. On `master@upstream` tip:

- `AppState::ViewState` is `{ terminal_area, pane_infos }`. `sidebar_rect`,
  `workspace_card_areas`, `tab_bar_rect`, `tab_hit_areas` and `ViewLayout` do not
  exist anywhere in `src/` — verified by a repo-wide grep for each identifier.
- `compute_workspace_card_areas`, `agent_panel_scroll_metrics`,
  `agent_panel_body_rect`, `agent_entry_height_in_body`, `agent_entry_gap` are
  all gone; `collapsed_sidebar_sections` now lives at
  `src/client/shell/sidebar.rs` as `pub(in crate::client::shell)`.
- Server `Mode` shrank to `Navigate | Terminal`. The five overlay modes the
  fork's `obscured` tested (`Onboarding`, `ReleaseNotes`, `Settings`,
  `KeybindHelp`, `Navigator`) are now client-side `ClientShellMode`/overlay
  state.
- Sidebar and tab-bar geometry now lives only in the TUI **client**:
  `ClientShellLayout { sidebar, tab_bar, mobile_header, pane_surface }`
  (`src/client/shell/state.rs:113`, computed in `src/client/shell/config.rs:350`)
  and `ShellHitMap { workspaces, agents, tabs, .. }`
  (`src/client/shell/state.rs:141`), stored as `self.hits` after each
  `compose()` (`src/client/shell/composition.rs:136`).

So every field of the `ui.layout` envelope is client-owned, and the server —
which is what answers the JSON API socket — knows none of it. The client socket
is *served by* the server (`src/server/socket_paths.rs`), so there is no way to
query a client; the data must travel client → server.

**Decision: publish the shell's layout client → server over the wire protocol,
and have `ui.layout` serve the last report.** Rejected alternatives:

- *Recompute server-side.* The layout is a pure function of surface size, config
  and snapshot **plus** client-local state (`sidebar_collapsed`,
  `sidebar_section_split`, scroll offsets, overlay mode) the server does not
  have. Recomputing would duplicate the layout and still be wrong.
- *Change the envelope to what the server can answer.* Forbidden by the root
  brief: modaliser is not edited and the wire shape is preserved verbatim.
- *Serve from the client.* No client-side listener exists.

Upstream already supplies the idiom this needs, three hops, used twice:
`ClientMessage::ClientShellFocus` / `ClientShellMouseCapture` →
`src/server/client_transport.rs:1133` maps to `ServerEvent::… { client_id, .. }`
→ handled in `src/server/headless.rs:2279`. The new report follows that path
rather than inventing a fourth.

**Consequence for `Done when`:** the 13 tests in the fork's `src/app/api/ui.rs`
cannot "come across" — they drive `crate::ui::compute_view` over `AppState`
sidebar geometry that no longer exists. Their *coverage* is re-homed to the
client-side report builder, which is where the geometry now is. The schema
round-trip test in `src/api/schema/tests.rs` does port verbatim.

**On the CLAUDE.md runtime/client guardrail:** this does push TUI presentation
geometry to the server, which the guardrail discourages for *new* shared
behaviour. It is accepted here because (a) the task file forbids relitigating
`ui.layout`'s straddle, (b) the root brief fixes the wire shape, and (c) the
server treats the report as opaque last-reported client data keyed by client, not
as state it reasons about. Nothing here is proposed upstream.

### The report rides `EndpointControl`, not a new `ClientMessage` variant

`CLAUDE.md`'s stable client endpoint contract makes generation-1 codec enums
append-closed — *"Do not add, remove, reorder, or reinterpret fields or enum
variants reachable from a published codec"* — and `wire.rs` freezes
`ClientMessage`'s tags in `client_message_wire_tags` (`EndpointControl` is 20).
A variant 21 would have been a contract break needing a new codec name.

`ClientMessage::EndpointControl { kind, data }` exists for exactly this: an
*"extensible named control message"* whose unknown kinds both sides ignore. The
in-tree precedent is `PRESENTATION_EFFECTS_SYNC_KIND`, which maps a named kind to
a `ServerEvent { client_id, .. }` in `src/server/client_transport.rs`.

So the report travels as `kind = "herdr.ui.layout.report.v1"` with a JSON body.
Consequences: **no `PROTOCOL_VERSION` bump, no wire-tag or frozen-fixture edits,
and `tests/fixtures/endpoint-*.json` are untouched.** An upstream server ignores
the kind rather than failing.

### Where each piece landed

- `src/client/shell/ui_layout.rs` — builds the report from `ShellHitMap` +
  `ClientShellLayout`, so reported rects are the rects last painted.
- `src/client/shell_runtime.rs::publish_client_shell_ui_layout` — one call site,
  once per client loop iteration, change-gated against
  `ClientState::published_ui_layout`. It is not hung off an input path because
  layout also moves on snapshot arrival and overlay changes.
- `src/server/client_transport.rs` / `src/server/headless.rs` — decode, then
  store on `AppState::ui_layout` only from the **foreground** active shell client,
  so several attached shells cannot fight over one answer.
- `src/app/api/ui.rs` — replays the stored report; with no shell attached it
  answers `UiLayoutReport::default()`, a well-formed envelope with no entries.

### Unpainted rects are dropped

The sidebar renderer emits degenerate zero-size hits for rows it clipped away —
at 80x0 it produced `ws_1` at `{x:0, y:2, width:0, height:0}`. The fork already
filtered tab hits on `rect.width > 0`; that filter is now applied to workspaces
and agents too. A consumer painting chips over a zero-size rect would place them
on nothing, so this preserves the fork's contract rather than extending it.

### Evidence

- **Wire shape byte-identical.** The 188 lines this adds to
  `docs/next/api/herdr-api.schema.json` `diff` clean against the 188 lines
  `linkuistics-fork` adds to the same file. Instrument checked: mutating one byte
  makes the diff dirty, so the clean read is not a blind comparison.
- **`cargo nextest run --locked --no-fail-fast`**: `3100 tests run: 3096 passed,
  4 failed, 2 skipped`. The four are `live_handoff_preserves_pane_process_io`,
  `live_handoff_keeps_agent_started_pane_after_agent_exits`,
  `live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session`,
  `pane_info_and_subscriptions_expose_done_agent_status` — item for item the
  baseline set in `02-DONE-impl--upstream-base-k2.md`, no additions.
- **Green:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `just windows-lint`, and all five script/doc gates (`maintenance-test` 101,
  `ui-hot-path-architecture-test` 6, `integration-assets-test` 5,
  `plugin-marketplace-test` 31, `docs-contract-test` 7).
- **Run for real.** A debug `herdr --session portcheck` under a 100x30 pty,
  queried with `herdr --session portcheck ui layout`, returned canvas 100x30,
  sidebar rect 26x30, workspace `w1` at `{0,2,25,1}`, tab bar `{26,0,74,1}` and
  tab `w1:t1` at `{26,0,8,1}`. Every field contradicts
  `UiLayoutReport::default()`, so the report crossed the whole path rather than
  falling back. `obscured` read `true` under onboarding and `false` after
  dismissing it, so the overlay mapping is driven, not stuck. Throwaway sessions
  removed; Antony's own server on `~/.config/herdr/herdr.sock` was not touched.

### Not done here — needs Antony

**modaliser has not been seen painting chips against this build.** That is the
task's stated real acceptance test and it needs a human at a terminal with
modaliser loaded. The envelope it consumes is proven byte-identical to the one it
consumes today, and a live query returns real geometry, so the risk is low — but
it is unverified, and it is the one `Done when` item this session cannot close.

### `linkuistics-fork-next` advanced to this commit

Same discipline `hook-authority-k3` recorded: the bookmark tracks the last
landed cluster port, so Antony's rename to `linkuistics-fork` after the grove
ends carries all of them. `jj bookmark set linkuistics-fork-next -r @-`.
