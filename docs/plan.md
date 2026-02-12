# VectorCade Web Yew - Implementation Plan

## Current State

The repo contains a skeleton Yew app with:
- Game picker dropdown (using `vectorcade_games::all_games()`)
- Canvas element (not wired)
- Basic HTML/CSS styling

**Compilation Status**: Failing - missing `web_sys` dependency, type inference issues

## Phase 1: Fix Compilation (Priority: High)

### Tasks
1. Add `web-sys` to Cargo.toml dependencies with required features
2. Fix `use_memo` type annotation in main.rs
3. Verify `cargo check` passes
4. Verify `trunk build` produces WASM bundle

### Acceptance Criteria
- `cargo check` succeeds
- `trunk serve` starts without errors

## Phase 2: Basic Rendering (Priority: High)

### Tasks
1. Create Canvas2D renderer (simpler than wgpu for MVP)
2. Wire renderer to canvas element via `web_sys::HtmlCanvasElement`
3. Implement `DrawCmd` rendering: `Clear`, `Line`, `Polyline`
4. Set up requestAnimationFrame loop

### Acceptance Criteria
- Can draw a single line on canvas
- Black background renders correctly

## Phase 3: Game Loop Integration (Priority: High)

### Tasks
1. Implement fixed timestep loop (60Hz updates)
2. Create `InputState` implementation from keyboard events
3. Wire game update/render cycle
4. Connect game selector to switch active game

### Acceptance Criteria
- Pong game runs and responds to input
- Game switching works via dropdown

## Phase 4: Input & Polish (Priority: Medium)

### Tasks
1. Map arrow keys / WASD to game controls
2. Implement Reset button functionality
3. Add pause (Escape key)
4. Basic touch controls (stretch goal)

### Acceptance Criteria
- All controls documented and working
- Reset restarts current game

## Phase 5: Additional Games (Priority: Medium)

### Tasks
1. Test Asteroids game
2. Implement Lunar Lander in vectorcade-games (if not done)
3. Implement Battlezone projection helpers
4. Implement Tempest (most complex)

### Acceptance Criteria
- 4-5 games selectable and playable

## Phase 6: Video-Ready Polish (Priority: Low)

### Tasks
1. Add vector font score rendering
2. Optional glow/CRT effects
3. Clean UI for recording
4. Test at various resolutions

### Acceptance Criteria
- Recording-quality visual output
- No visual glitches during play

## Dependencies on Other Repos

| Repo | Status | Needed |
|------|--------|--------|
| vectorcade-shared | Complete | Game trait, DrawCmd |
| vectorcade-games | In Progress | Pong, Asteroids done; need more games |
| vectorcade-render-wgpu | Stub only | NullRenderer exists; real renderer needed |
| vectorcade-fonts | Complete | Multiple font styles available |

## Next Immediate Steps

1. Fix compilation errors (add web_sys, fix types)
2. Test `trunk serve` locally
3. Wire Canvas2D basic rendering
4. Get Pong running in browser
