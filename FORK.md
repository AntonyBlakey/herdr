# Linkuistics fork maintenance

This file belongs to the `linkuistics-fork` branch of `AntonyBlakey/herdr`.
Upstream does not have it. It records how agents keep the fork current, release it and install it.
Every agent and harness that works in this checkout reads it through the pointer at the end of `AGENTS.md`.

## What the fork is

The fork carries three patches on top of upstream master:

- `ui.layout`: an API method and CLI verb that report the drawn sidebar and tab-bar geometry. Modaliser uses it.
- Hook authority: session identity is separate from lifecycle state in hook reports.
- Pane appearance: configurable pane border colours and an inactive-pane haze. Both are off by default.

The fork is not a staging area for upstream contributions.
Do not open upstream issues or pull requests for these patches.
The account that works here is not an upstream maintainer, so the maintainer workflow in `AGENTS.md` does not apply.

## Repository layout

- The checkout is a colocated jj repository. Use jj for every mutation. Use git only to read.
- Remote `upstream` is `ogulcancelik/herdr`. Remote `origin` is `AntonyBlakey/herdr`.
- Bookmark `linkuistics-fork` holds the fork commits on top of `master@upstream`.
- Bookmark `master` mirrors `master@upstream`. Push it to `origin` when you sync.

## Sync with upstream

1. Fetch: `jj git fetch --remote upstream`.
2. Move the mirror: `jj bookmark set master -r master@upstream`.
3. Rebase the fork commits: `jj rebase -s <first fork commit> -d master@upstream`.
4. Resolve each conflicted commit in a child change, then run `jj squash` into it. Start at the oldest commit.
5. Validate the tip (see below).
6. Push: `jj git push --remote origin -b linkuistics-fork -b master`. The rebase rewrites commits, so this push moves the bookmarks sideways.

Upstream often restructures the files that the patches touch.
When a hunk conflicts, keep upstream's new structure and add the fork's change to it.
For example, upstream now declares API methods in the `define_operations!` table in `src/api/schema.rs`.
`ui.layout` is one entry there, with both flags `false`.

## Validate

Run `just check` on the tip. This machine needs two overrides:

- Set `ZIG` to a Zig 0.16.0 binary. Homebrew's `zig` is newer, and the vendored libghostty-vt build refuses it.
- Put `$HOME/.cargo/bin` first on `PATH`. Homebrew's `cargo` otherwise comes first and ignores `rust-toolchain.toml`. Its newer clippy then fails on upstream code.

`api_ping::pane_info_and_subscriptions_expose_done_agent_status` times out on this machine.
It also fails on pure upstream master (2026-10-21), so it is not a fork regression.
Before you accept any other failure, compare it with upstream in a separate jj workspace.

## Release and install

A release is a bump of the Homebrew formula. There is no GitHub release.

1. Edit `Formula/linkuistics-herdr.rb` in the project checkout `~/Development/homebrew-taps`. Do not edit the Homebrew tap clone under `/opt/homebrew/Library/Taps/`.
2. Set `revision` to the pushed `linkuistics-fork` commit.
3. Set `version` to `<upstream Cargo version>-linkuistics.<seq>`. Increase `<seq>` on every release. When the upstream version changes, set `<seq>` to 1.
4. Describe the change as `linkuistics-herdr v<version>`, move bookmark `main` to it and push `main`.
5. Run `brew update`.
6. Run `brew upgrade linkuistics/taps/linkuistics-herdr`.
7. Run `herdr --version` and make sure that it prints the new version.

A running herdr server keeps the old binary until it restarts.
