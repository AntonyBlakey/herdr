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
