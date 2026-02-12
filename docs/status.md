# VectorCade Web Yew - Project Status

**Last Updated**: 2026-02-12

## Overall Status: Skeleton / Pre-MVP

The repo contains initial scaffolding but is not yet functional.

## Build Status

| Check | Status |
|-------|--------|
| `cargo check` | FAILING |
| `trunk build` | NOT TESTED |
| `trunk serve` | NOT TESTED |

### Current Compilation Errors

1. **Missing dependency**: `web_sys` not in Cargo.toml
   - Error: `use of unresolved module or unlinked crate 'web_sys'`
   - Fix: Add `web-sys = { version = "0.3", features = ["HtmlSelectElement"] }`

2. **Type inference**: `use_memo` needs explicit type
   - Error: `type must be known at this point`
   - Fix: `let games: Rc<Vec<Box<dyn Game + Send>>> = use_memo(...)`

## Component Status

| Component | Status | Notes |
|-----------|--------|-------|
| Yew App Shell | Skeleton | Compiles with fixes |
| Game Picker | Partial | UI exists, needs type fix |
| Canvas Element | Stub | In HTML, not wired to Rust |
| Game Loop | Not Started | - |
| Input Handling | Not Started | - |
| Renderer Integration | Not Started | - |
| Touch Controls | Not Started | - |

## Dependency Repos Status

| Repo | Status | Games/Features |
|------|--------|----------------|
| vectorcade-shared | Complete | DrawCmd, Game trait, input API |
| vectorcade-core | Complete | Rgba, RNG |
| vectorcade-math | Complete | Collision, projection |
| vectorcade-fonts | Complete | 5 font styles (Atari, Midway, etc.) |
| vectorcade-games | In Progress | Pong, Asteroids implemented |
| vectorcade-render-wgpu | Stub | NullRenderer only |

## Games Available

| Game | Implemented | Tested in Web |
|------|-------------|---------------|
| Pong | Yes (vectorcade-games) | No |
| Asteroids | Yes (vectorcade-games) | No |
| Lunar Lander | No | - |
| Battlezone | No | - |
| Tempest | No | - |

## Blockers

1. **Compilation errors** must be fixed before any runtime testing
2. **No renderer implementation** - NullRenderer doesn't draw anything
3. **Game loop not wired** - no update/render cycle

## Immediate Next Steps

1. Fix Cargo.toml (add web-sys dependency)
2. Fix main.rs type annotation
3. Verify compilation
4. Implement basic Canvas2D renderer
5. Wire game loop

## Milestones

- [ ] Project compiles
- [ ] trunk serve works
- [ ] Single game renders on canvas
- [ ] Input controls work
- [ ] All games selectable
- [ ] Video-recording ready
