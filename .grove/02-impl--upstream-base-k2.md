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
