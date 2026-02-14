# vectorcade-web-yew

A Yew-based web shell for playing vector arcade games in the browser using WebGPU/wgpu rendering with 4x MSAA anti-aliasing.

![VectorCade Screenshot](images/screenshot.png?ts=1739497200)

**[▶ Play Live Demo](https://softwarewrighter.github.io/vectorcade-web-yew/)**

## Features

- **WebGPU Rendering** - Hardware-accelerated GPU rendering via wgpu with 4x MSAA anti-aliasing
- **Vector Fonts** - All text rendered using custom vector fonts (no raster fonts)
- **Multiple Games** - Pong, Asteroids, Lunar Lander, Battlezone, Tempest, and more
- **60Hz Game Loop** - Fixed timestep physics with smooth rendering
- **Keyboard Controls** - Arrow keys, WASD, and Space for game input
- **Game Picker** - Dropdown menu to switch between games
- **About Dialog** - Build info, links, and license information

## Architecture

This repo is part of the vectorcade multi-repo DAG:

```
vectorcade-shared (core types)
       ↓
   ┌───┴───┐
   ↓       ↓
vectorcade-fonts  vectorcade-games
   ↓       ↓
   └───┬───┘
       ↓
vectorcade-web-yew (this repo)
```

## Controls

| Key | Action |
|-----|--------|
| ↑ / W | Thrust / Up |
| ↓ / S | Down |
| ← / A | Rotate Left |
| → / D | Rotate Right |
| Space | Fire / Action |

## Development

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) for WASM bundling

### Setup

Clone all vectorcade repos into a common parent directory:

```bash
mkdir vectorcade && cd vectorcade
git clone <vectorcade-shared>
git clone <vectorcade-fonts>
git clone <vectorcade-games>
git clone <vectorcade-web-yew>
```

### Run Development Server

```bash
cd vectorcade-web-yew/vectorcade-web-yew
trunk serve --port 8714 --open
```

### Build for Production

```bash
trunk build --release
```

Build artifacts are placed in `dist/`. For GitHub Pages deployment, copy to `pages/` and fix paths to be relative.

## Deployment

The `pages/` directory contains pre-built WASM artifacts for GitHub Pages. The `.github/workflows/pages.yml` workflow automatically deploys on push to main.

## License

See [LICENSE](LICENSE) and [COPYRIGHT](COPYRIGHT) files.
