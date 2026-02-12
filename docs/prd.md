# VectorCade Web Yew - Product Requirements Document

## Vision

A browser-based vector graphics arcade platform showcasing 4-5 classic vector-style games, suitable for creating demo videos and blog content.

## Goals

1. **Playable Demos**: Display 4-5 classic vector arcade game demos (Pong, Asteroids, Lunar Lander, Battlezone, Tempest)
2. **Video Recording**: Serve a web UI via trunk that can be screen-recorded for an explainer video
3. **Blog Publishing**: Support creation of a blog post about the vectorcade repos

## User Stories

### As a viewer/player
- I want to select a game from a dropdown menu
- I want to see vector-style graphics (lines, polylines) on a black background
- I want to control the game with keyboard (and optionally touch controls)
- I want to see my score rendered in a vector font style

### As a content creator
- I want to run the demo locally via `trunk serve`
- I want to record gameplay footage for video content
- I want visually distinct games with period-appropriate aesthetics

## Functional Requirements

### Must Have (MVP)
1. Game picker dropdown with available games
2. Canvas rendering of DrawCmd display lists
3. Keyboard input mapped to game controls
4. Fixed timestep game loop
5. At least 2 working games (Pong, Asteroids)

### Should Have
1. Touch controls overlay for mobile
2. Pause/Reset functionality
3. Score display using vector fonts
4. 3 additional games (Lunar Lander, Battlezone, Tempest)

### Nice to Have
1. PWA manifest for installable web app
2. Multiple vector font styles per game
3. CRT/phosphor glow effects
4. Sound effects

## Non-Functional Requirements

1. **Performance**: 60fps rendering with 50-200 vector segments
2. **Compatibility**: Modern browsers with WebGPU/WebGL support
3. **Portability**: Graceful fallback for unsupported browsers
4. **Build Size**: Optimized WASM bundle (LTO, opt-level "s")

## Success Criteria

1. All 4-5 games playable in browser
2. Smooth enough for video recording
3. Visually appealing for blog screenshots
4. Runs via `trunk serve --open`
