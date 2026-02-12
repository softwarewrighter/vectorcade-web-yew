# Agent Guide

This repo is part of a multi-repo DAG:

- `vectorcade-shared` (root) → shared types + traits
- `vectorcade-fonts` → vector glyph/stroke fonts (depends on shared)
- `vectorcade-games` → game logic crates (depends on shared + fonts)
- `vectorcade-render-wgpu` → renderer backend (depends on shared)
- `vectorcade-web-yew` → integration app (depends on shared + games + renderer)

## Rules of engagement

- Keep `vectorcade-shared` pure (no wasm/webgpu/web-sys).
- Prefer deterministic logic (fixed timestep, seeded RNG) in games.
- All rendering must go through `DrawCmd` (display-list). No game touches wgpu.

## Local multi-repo dev (recommended)

Clone all repos into the same parent directory:

```
arcade/
  vectorcade-shared/
  vectorcade-fonts/
  vectorcade-games/
  vectorcade-render-wgpu/
  vectorcade-web-yew/
```

Each repo includes a `.cargo/config.toml` with `[patch.crates-io]` entries
that will automatically route `vectorcade-*` deps to local checkouts when present.

If you prefer git deps, remove/ignore that config and set the `git = "..."`
URLs in `Cargo.toml`.
