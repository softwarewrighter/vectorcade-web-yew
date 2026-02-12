# vectorcade-web-yew

Yew-based web shell that hosts the canvas, game picker, and mobile controls.

This repo integrates:
- `vectorcade-shared`
- `vectorcade-games`
- `vectorcade-render-wgpu`

## Dev

This is a skeleton. Agent should:
- add `trunk` setup
- wire input events → `InputState`
- run fixed timestep loop
- connect renderer backend

## Local multi-repo dev

Clone all repos into one parent (see `AGENTS.md`). Then:

- Install trunk: `cargo install trunk`
- Run: `trunk serve --open`

(Agent will flesh out config and wasm target settings.)
