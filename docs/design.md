# VectorCade Web Yew - Design Document

## UI Design

### Layout
```
┌─────────────────────────────────────────────────┐
│ VectorCade (skeleton)          [Game ▼] [Reset] │
│                                                  │
│                                                  │
│                    GAME CANVAS                   │
│                  (black background,              │
│                   vector graphics)               │
│                                                  │
│                                                  │
└─────────────────────────────────────────────────┘
```

### Components
- **HUD**: Title text (top-left)
- **Panel**: Game selector + reset button (top-right)
- **Canvas**: Full viewport game rendering area

### Styling
- Black background throughout
- Monospace font for UI elements
- Minimal chrome to maximize game visibility

## Technical Design

### Game Loop Architecture

```rust
// Pseudocode for fixed timestep loop
const TIMESTEP: f32 = 1.0 / 60.0;
let mut accumulator = 0.0;

fn tick(dt: f32, game: &mut dyn Game, ctx: &mut GameCtx) {
    accumulator += dt;
    while accumulator >= TIMESTEP {
        game.update(ctx, TIMESTEP);
        accumulator -= TIMESTEP;
    }
    let mut cmds = Vec::new();
    game.render(ctx, &mut cmds);
    renderer.draw(&cmds);
}
```

### Input Handling

Map browser events to `InputState`:

| Browser Event | InputState |
|--------------|------------|
| `keydown` | `key(Key::X).pressed()` |
| `keyup` | `key(Key::X).released()` |
| `touchstart` | `pointer().Some` |
| `touchmove` | `axis(Axis::X)` |

### Rendering Pipeline

1. Game calls `render()` → outputs `Vec<DrawCmd>`
2. DrawCmd types: `Clear`, `Line`, `Polyline`, `Text`, `Transform`
3. Renderer tessellates lines into triangles (via lyon)
4. WebGPU/Canvas2D draws to canvas

### State Management

Using Yew hooks:
- `use_state` for selected game index
- `use_memo` for game list (loaded once)
- `use_effect` for game loop setup/teardown

## Module Structure

```
vectorcade-web-yew/
├── src/
│   ├── main.rs          # App component, game picker
│   ├── canvas.rs        # Canvas component (TODO)
│   ├── input.rs         # InputState impl (TODO)
│   ├── game_loop.rs     # Fixed timestep loop (TODO)
│   └── renderer.rs      # Renderer integration (TODO)
├── index.html
└── Trunk.toml
```

## Integration Patterns

### Resolving Local Dependencies

`.cargo/config.toml`:
```toml
[patch.crates-io]
vectorcade-shared = { path = "../vectorcade-shared/vectorcade-shared" }
vectorcade-fonts = { path = "../vectorcade-fonts/vectorcade-fonts" }
vectorcade-games = { path = "../vectorcade-games/vectorcade-games" }
vectorcade-render-wgpu = { path = "../vectorcade-render-wgpu/vectorcade-render-wgpu" }
```

### Game Registration

Games expose metadata and implement the `Game` trait:
```rust
pub trait Game {
    fn metadata(&self) -> GameMeta;
    fn reset(&mut self, ctx: &mut GameCtx);
    fn update(&mut self, ctx: &mut GameCtx, dt: f32);
    fn render(&mut self, ctx: &mut GameCtx, out: &mut Vec<DrawCmd>);
}
```
