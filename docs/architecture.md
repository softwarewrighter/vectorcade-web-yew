# VectorCade Web Yew Architecture

## Overview

VectorCade Web Yew is the browser-facing integration layer that hosts vector arcade games in a Yew-based web shell. It is the "top" of the multi-repo DAG.

## Repo DAG (Dependencies)

```
vectorcade-core (pure math, RNG, Rgba)
        │
        ▼
vectorcade-math (collision, projection helpers)
        │
        ▼
vectorcade-shared (API contracts: DrawCmd, Game trait, input)
        │
        ├──────────────────┬─────────────────────┐
        ▼                  ▼                     ▼
vectorcade-fonts    vectorcade-games    vectorcade-render-wgpu
(vector font          (Pong, Asteroids,   (NullRenderer stub,
 styles)               Lunar Lander...)    wgpu+lyon planned)
        │                  │                     │
        └──────────────────┴─────────────────────┘
                           │
                           ▼
               vectorcade-web-yew (this repo)
                    (Yew shell, canvas host, input wiring)
```

## Key Modules

### Yew Shell (src/main.rs)
- Game picker UI (dropdown select)
- Canvas element host
- Reset button (stub)
- HUD overlay

### Dependencies Used
- `yew` v0.21 - Reactive web framework (CSR mode)
- `wasm-bindgen` - Rust/JS interop
- `gloo` - Browser API bindings
- `web-sys` - (needed, currently missing from Cargo.toml)

## Data Flow

```
User Input  →  InputState trait  →  Game::update()
                                         │
                                         ▼
                              Game::render() → Vec<DrawCmd>
                                         │
                                         ▼
                                VectorRenderer::draw()
                                         │
                                         ▼
                                  Canvas/WebGPU
```

## Integration Points

1. **Game Registry**: `vectorcade_games::all_games()` returns all available games
2. **Game Trait**: Each game implements `vectorcade_shared::game::Game`
3. **Draw Commands**: Games emit `DrawCmd` display lists
4. **Renderer**: `vectorcade_render_wgpu::VectorRenderer` trait (currently NullRenderer)

## Local Development Setup

All repos must be cloned as siblings:
```
sw-fun/
  vectorcade-shared/
  vectorcade-fonts/
  vectorcade-games/
  vectorcade-render-wgpu/
  vectorcade-web-yew/   ← this repo
```

The `.cargo/config.toml` uses `[patch.crates-io]` to resolve dependencies from local paths.

## Build Target

- Primary: WebAssembly (wasm32-unknown-unknown)
- Tooling: trunk (for WASM bundling and dev server)
