//! VectorCade Web - Yew-based vector arcade platform.
//!
//! This module provides the browser shell that hosts vector arcade games
//! using Canvas2D rendering and keyboard/touch input.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};
use yew::prelude::*;

use vectorcade_fonts::{AtariMini, Cinematronics, FontRegistry, Midway, VectorScanline};
use vectorcade_games::all_games;
use vectorcade_shared::draw::{DrawCmd, Stroke};
use vectorcade_shared::font::{FontStyleId, GlyphPathCmd};
use vectorcade_shared::game::{AudioOut, Game, GameCtx, GameMeta, ScreenInfo};
use vectorcade_shared::input::{Axis, Button, InputState, Key, Pointer};
use vectorcade_shared::{Rgba, Xorshift64};

/// Fixed timestep for game updates (60 Hz).
const TIMESTEP: f32 = 1.0 / 60.0;

/// Keyboard input state tracking.
#[derive(Default)]
struct WebInput {
    keys: HashMap<Key, bool>,
    prev_keys: HashMap<Key, bool>,
}

impl WebInput {
    fn set_key(&mut self, key: Key, down: bool) {
        self.keys.insert(key, down);
    }

    fn end_frame(&mut self) {
        self.prev_keys = self.keys.clone();
    }

    fn map_code(code: &str) -> Option<Key> {
        match code {
            "ArrowLeft" | "KeyA" => Some(Key::Left),
            "ArrowRight" | "KeyD" => Some(Key::Right),
            "ArrowUp" | "KeyW" => Some(Key::Up),
            "ArrowDown" | "KeyS" => Some(Key::Down),
            "Space" => Some(Key::Space),
            "Enter" => Some(Key::Enter),
            "Escape" => Some(Key::Escape),
            "KeyZ" => Some(Key::Z),
            "KeyX" => Some(Key::X),
            "KeyC" => Some(Key::C),
            _ => None,
        }
    }
}

impl InputState for WebInput {
    fn key(&self, k: Key) -> Button {
        let is_down = *self.keys.get(&k).unwrap_or(&false);
        let was_down = *self.prev_keys.get(&k).unwrap_or(&false);
        Button {
            is_down,
            went_down: is_down && !was_down,
            went_up: !is_down && was_down,
        }
    }

    fn axis(&self, a: Axis) -> f32 {
        match a {
            Axis::MoveX => {
                let left = if self.key(Key::Left).is_down {
                    -1.0
                } else {
                    0.0
                };
                let right = if self.key(Key::Right).is_down {
                    1.0
                } else {
                    0.0
                };
                left + right
            }
            Axis::MoveY => {
                let up = if self.key(Key::Up).is_down { 1.0 } else { 0.0 };
                let down = if self.key(Key::Down).is_down {
                    -1.0
                } else {
                    0.0
                };
                up + down
            }
            Axis::Thrust => {
                if self.key(Key::Up).is_down || self.key(Key::W).is_down {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    fn pointer(&self) -> Option<Pointer> {
        None // TODO: mouse/touch support
    }
}

/// Stub audio output.
struct WebAudio;
impl AudioOut for WebAudio {}

/// Global glow intensity multiplier (0.0 = off, 1.0 = full CRT effect).
const GLOW_INTENSITY: f64 = 0.8;

/// Apply CRT phosphor glow effect to the canvas context.
fn apply_glow(ctx: &CanvasRenderingContext2d, color: &Rgba, glow: f32) {
    if glow > 0.0 && GLOW_INTENSITY > 0.0 {
        let blur = (8.0 + glow as f64 * 12.0) * GLOW_INTENSITY;
        ctx.set_shadow_blur(blur);
        ctx.set_shadow_color(&rgba_to_css_glow(color, 0.6 * glow * GLOW_INTENSITY as f32));
    } else {
        ctx.set_shadow_blur(0.0);
    }
}

/// Clear glow effect.
fn clear_glow(ctx: &CanvasRenderingContext2d) {
    ctx.set_shadow_blur(0.0);
}

/// Render DrawCmd list to Canvas2D with CRT phosphor glow effects.
fn render_to_canvas(
    ctx: &CanvasRenderingContext2d,
    cmds: &[DrawCmd],
    width: f64,
    height: f64,
    fonts: &FontRegistry,
) {
    let scale = width.min(height) / 2.0;
    let cx = width / 2.0;
    let cy = height / 2.0;

    // Transform from NDC [-1,1] to canvas pixels
    let to_px =
        |x: f32, y: f32| -> (f64, f64) { (cx + (x as f64) * scale, cy - (y as f64) * scale) };

    for cmd in cmds {
        match cmd {
            DrawCmd::Clear { color } => {
                clear_glow(ctx);
                ctx.set_fill_style_str(&rgba_to_css(color));
                ctx.fill_rect(0.0, 0.0, width, height);
            }
            DrawCmd::Line(line) => {
                let (x1, y1) = to_px(line.a.x, line.a.y);
                let (x2, y2) = to_px(line.b.x, line.b.y);
                draw_line_with_glow(ctx, x1, y1, x2, y2, &line.stroke);
            }
            DrawCmd::Polyline {
                pts,
                closed,
                stroke,
            } => {
                if pts.len() < 2 {
                    continue;
                }
                draw_polyline_with_glow(ctx, pts, *closed, stroke, &to_px);
            }
            DrawCmd::Text {
                pos,
                text,
                size_px,
                color,
                style,
            } => {
                render_vector_text_with_glow(
                    ctx, fonts, *style, text, pos.x, pos.y, *size_px, color, scale, cx, cy,
                );
            }
            // Transform stack not implemented for Canvas2D MVP
            DrawCmd::PushTransform(_) | DrawCmd::PopTransform => {}
            DrawCmd::BeginLayer { .. } | DrawCmd::EndLayer => {}
        }
    }

    // Ensure glow is cleared at end
    clear_glow(ctx);
}

/// Render text using vector fonts with CRT glow effect.
fn render_vector_text_with_glow(
    ctx: &CanvasRenderingContext2d,
    fonts: &FontRegistry,
    style: FontStyleId,
    text: &str,
    x: f32,
    y: f32,
    size_px: f32,
    color: &Rgba,
    scale: f64,
    cx: f64,
    cy: f64,
) {
    // Get font, fall back to default if style not found
    let font = fonts
        .get(style)
        .or_else(|| fonts.get(FontStyleId::DEFAULT))
        .or_else(|| fonts.get(FontStyleId::ATARI));

    let Some(font) = font else {
        // No fonts available, skip rendering
        return;
    };

    // Apply glow for text
    apply_glow(ctx, color, 0.6);

    ctx.set_stroke_style_str(&rgba_to_css(color));
    ctx.set_line_width(2.0);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    let mut cursor_x = x;
    let glyph_scale = size_px / scale as f32; // Scale factor for glyphs

    for ch in text.chars() {
        if !font.has_glyph(ch) {
            // Advance cursor for missing glyphs (space-like)
            cursor_x += glyph_scale * 0.6;
            continue;
        }

        let paths = font.glyph_paths(ch);
        for path in paths {
            ctx.begin_path();
            let mut path_started = false;

            for cmd in &path.cmds {
                match cmd {
                    GlyphPathCmd::MoveTo(pt) => {
                        let px = cx + ((cursor_x + pt.x * glyph_scale) as f64) * scale;
                        let py = cy - ((y + pt.y * glyph_scale) as f64) * scale;
                        ctx.move_to(px, py);
                        path_started = true;
                    }
                    GlyphPathCmd::LineTo(pt) => {
                        if !path_started {
                            let px = cx + ((cursor_x + pt.x * glyph_scale) as f64) * scale;
                            let py = cy - ((y + pt.y * glyph_scale) as f64) * scale;
                            ctx.move_to(px, py);
                            path_started = true;
                        } else {
                            let px = cx + ((cursor_x + pt.x * glyph_scale) as f64) * scale;
                            let py = cy - ((y + pt.y * glyph_scale) as f64) * scale;
                            ctx.line_to(px, py);
                        }
                    }
                    GlyphPathCmd::Close => {
                        ctx.close_path();
                    }
                }
            }
            ctx.stroke();
        }

        cursor_x += font.advance(ch) * glyph_scale;
    }

    clear_glow(ctx);
}

/// Draw a polyline with CRT glow effect.
fn draw_polyline_with_glow<F>(
    ctx: &CanvasRenderingContext2d,
    pts: &[glam::Vec2],
    closed: bool,
    stroke: &Stroke,
    to_px: &F,
) where
    F: Fn(f32, f32) -> (f64, f64),
{
    // Apply glow based on stroke settings
    let effective_glow = if stroke.glow > 0.0 {
        stroke.glow
    } else {
        0.5 // Default subtle glow for all lines
    };
    apply_glow(ctx, &stroke.color, effective_glow);

    ctx.begin_path();
    let (x0, y0) = to_px(pts[0].x, pts[0].y);
    ctx.move_to(x0, y0);
    for pt in pts.iter().skip(1) {
        let (x, y) = to_px(pt.x, pt.y);
        ctx.line_to(x, y);
    }
    if closed {
        ctx.close_path();
    }
    ctx.set_stroke_style_str(&rgba_to_css(&stroke.color));
    ctx.set_line_width(stroke.width_px as f64);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    ctx.stroke();

    clear_glow(ctx);
}

/// Draw a line with CRT glow effect.
fn draw_line_with_glow(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    stroke: &Stroke,
) {
    // Apply glow based on stroke settings
    let effective_glow = if stroke.glow > 0.0 {
        stroke.glow
    } else {
        0.5 // Default subtle glow for all lines
    };
    apply_glow(ctx, &stroke.color, effective_glow);

    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.line_to(x2, y2);
    ctx.set_stroke_style_str(&rgba_to_css(&stroke.color));
    ctx.set_line_width(stroke.width_px as f64);
    ctx.set_line_cap("round");
    ctx.stroke();

    clear_glow(ctx);
}

fn rgba_to_css(c: &Rgba) -> String {
    format!(
        "rgba({},{},{},{})",
        (c.0 * 255.0) as u8,
        (c.1 * 255.0) as u8,
        (c.2 * 255.0) as u8,
        c.3
    )
}

/// Convert RGBA to CSS with modified alpha for glow effect.
fn rgba_to_css_glow(c: &Rgba, alpha_mult: f32) -> String {
    format!(
        "rgba({},{},{},{})",
        (c.0 * 255.0) as u8,
        (c.1 * 255.0) as u8,
        (c.2 * 255.0) as u8,
        (c.3 * alpha_mult).min(1.0)
    )
}

/// Create a font registry with all available fonts.
fn create_font_registry() -> FontRegistry {
    let mut registry = FontRegistry::new();
    registry.register(AtariMini);
    registry.register(Cinematronics);
    registry.register(Midway);
    registry.register(VectorScanline);
    registry
}

/// Game state held outside Yew for the animation loop.
struct GameState {
    games: Vec<Box<dyn Game + Send>>,
    selected: usize,
    input: WebInput,
    rng: Xorshift64,
    accumulator: f32,
    last_time: f64,
    draw_cmds: Vec<DrawCmd>,
    screen: ScreenInfo,
    fonts: FontRegistry,
}

impl GameState {
    fn new() -> Self {
        Self {
            games: all_games(),
            selected: 0,
            input: WebInput::default(),
            rng: Xorshift64::new(42),
            accumulator: 0.0,
            last_time: 0.0,
            draw_cmds: Vec::with_capacity(1024),
            screen: ScreenInfo::default(),
            fonts: create_font_registry(),
        }
    }

    fn tick(&mut self, now: f64) {
        if self.last_time == 0.0 {
            self.last_time = now;
        }
        let dt = ((now - self.last_time) / 1000.0) as f32;
        self.last_time = now;
        self.accumulator += dt.min(0.25); // cap to avoid spiral of death

        let audio = WebAudio;
        while self.accumulator >= TIMESTEP {
            let mut ctx = GameCtx {
                input: &self.input,
                audio: &audio,
                rng: &mut self.rng,
                screen: self.screen,
                now_s: now / 1000.0,
            };
            if let Some(game) = self.games.get_mut(self.selected) {
                game.update(&mut ctx, TIMESTEP);
            }
            self.accumulator -= TIMESTEP;
        }

        self.draw_cmds.clear();
        let mut ctx = GameCtx {
            input: &self.input,
            audio: &audio,
            rng: &mut self.rng,
            screen: self.screen,
            now_s: now / 1000.0,
        };
        if let Some(game) = self.games.get_mut(self.selected) {
            game.render(&mut ctx, &mut self.draw_cmds);
        }

        self.input.end_frame();
    }

    fn select_game(&mut self, idx: usize) {
        if idx < self.games.len() && idx != self.selected {
            self.selected = idx;
            let audio = WebAudio;
            let mut ctx = GameCtx {
                input: &self.input,
                audio: &audio,
                rng: &mut self.rng,
                screen: self.screen,
                now_s: 0.0,
            };
            if let Some(game) = self.games.get_mut(self.selected) {
                game.reset(&mut ctx);
            }
        }
    }

    fn reset_current(&mut self) {
        let audio = WebAudio;
        let mut ctx = GameCtx {
            input: &self.input,
            audio: &audio,
            rng: &mut self.rng,
            screen: self.screen,
            now_s: 0.0,
        };
        if let Some(game) = self.games.get_mut(self.selected) {
            game.reset(&mut ctx);
        }
    }

    fn game_metadata(&self) -> Vec<GameMeta> {
        self.games.iter().map(|g| g.metadata()).collect()
    }
}

thread_local! {
    static GAME_STATE: RefCell<GameState> = RefCell::new(GameState::new());
}

#[function_component(App)]
fn app() -> Html {
    let canvas_ref = use_node_ref();
    let selected = use_state(|| 0usize);

    // Get game metadata for the dropdown
    let game_meta: Vec<GameMeta> = GAME_STATE.with(|state| state.borrow().game_metadata());

    // Setup animation loop on mount
    {
        let canvas_ref = canvas_ref.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("no window");
            let document = window.document().expect("no document");

            // Setup keyboard listeners
            let keydown = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
                if let Some(key) = WebInput::map_code(&e.code()) {
                    GAME_STATE.with(|state| {
                        state.borrow_mut().input.set_key(key, true);
                    });
                    e.prevent_default();
                }
            });
            let keyup = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
                if let Some(key) = WebInput::map_code(&e.code()) {
                    GAME_STATE.with(|state| {
                        state.borrow_mut().input.set_key(key, false);
                    });
                }
            });
            document
                .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
                .unwrap();
            document
                .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
                .unwrap();
            keydown.forget();
            keyup.forget();

            // Start animation loop
            start_animation_loop(canvas_ref);

            || {}
        });
    }

    let on_change = {
        let selected = selected.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let idx = target.value().parse::<usize>().unwrap_or(0);
            selected.set(idx);
            GAME_STATE.with(|state| {
                state.borrow_mut().select_game(idx);
            });
        })
    };

    let on_reset = Callback::from(move |_| {
        GAME_STATE.with(|state| {
            state.borrow_mut().reset_current();
        });
    });

    html! {
        <div style="position: relative; width: 100%; height: 100%;">
            <div class="hud">{ "VectorCade" }</div>
            <div class="panel">
                <select onchange={on_change}>
                    { for game_meta.iter().enumerate().map(|(i, g)| html!{
                        <option value={i.to_string()} selected={*selected == i}>
                            { g.name }
                        </option>
                    })}
                </select>
                <button onclick={on_reset}>{ "Reset" }</button>
            </div>
            <canvas ref={canvas_ref} id="vectorcade-canvas"></canvas>
        </div>
    }
}

fn start_animation_loop(canvas_ref: NodeRef) {
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let canvas_ref = canvas_ref.clone();
    *g.borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        // Get canvas and context
        if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
            let window = web_sys::window().expect("no window");

            // Resize canvas to match display size
            let dpr = window.device_pixel_ratio();
            let rect = canvas.get_bounding_client_rect();
            let display_width = (rect.width() * dpr) as u32;
            let display_height = (rect.height() * dpr) as u32;

            if canvas.width() != display_width || canvas.height() != display_height {
                canvas.set_width(display_width);
                canvas.set_height(display_height);
            }

            // Update screen info and tick game
            GAME_STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.screen = ScreenInfo {
                    width_px: display_width,
                    height_px: display_height,
                    dpi_scale: dpr as f32,
                };
                state.tick(timestamp);

                // Render
                if let Ok(Some(ctx)) = canvas.get_context("2d") {
                    let ctx: CanvasRenderingContext2d = ctx.unchecked_into();
                    render_to_canvas(
                        &ctx,
                        &state.draw_cmds,
                        display_width as f64,
                        display_height as f64,
                        &state.fonts,
                    );
                }
            });
        }

        // Schedule next frame
        let window = web_sys::window().expect("no window");
        window
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }));

    // Start the loop
    let window = web_sys::window().expect("no window");
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
