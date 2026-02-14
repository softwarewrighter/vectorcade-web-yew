//! VectorCade Web - Yew-based vector arcade platform.
//!
//! This module provides the browser shell that hosts vector arcade games
//! using wgpu rendering with 4x MSAA and keyboard/touch input.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, KeyboardEvent};
use yew::prelude::*;

// Note: FontRegistry is now handled internally by WgpuRenderer
use vectorcade_games::all_games;
use vectorcade_render_wgpu::{VectorRenderer, WgpuRenderer};
use vectorcade_shared::draw::DrawCmd;
use vectorcade_shared::game::{AudioOut, Game, GameCtx, GameMeta, ScreenInfo};
use vectorcade_shared::input::{Axis, Button, InputState, Key, Pointer};
use vectorcade_shared::Xorshift64;

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
            "ArrowLeft" => Some(Key::Left),
            "ArrowRight" => Some(Key::Right),
            "ArrowUp" => Some(Key::Up),
            "ArrowDown" => Some(Key::Down),
            "KeyW" => Some(Key::W),
            "KeyA" => Some(Key::Left),
            "KeyS" => Some(Key::S),
            "KeyD" => Some(Key::Right),
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
        let mut did_update = false;
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
            did_update = true;
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

        // Only clear input state if we actually ran an update
        // This prevents losing key events when accumulator < TIMESTEP
        if did_update {
            self.input.end_frame();
        }
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
    static RENDERER: RefCell<Option<WgpuRenderer>> = const { RefCell::new(None) };
}

#[function_component(App)]
fn app() -> Html {
    let canvas_ref = use_node_ref();
    let selected = use_state(|| 0usize);
    let renderer_ready = use_state(|| false);

    // Get game metadata for the dropdown
    let game_meta: Vec<GameMeta> = GAME_STATE.with(|state| state.borrow().game_metadata());

    // Setup renderer and animation loop on mount
    {
        let canvas_ref = canvas_ref.clone();
        let renderer_ready = renderer_ready.clone();
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

            // Initialize wgpu renderer asynchronously
            let canvas_ref_clone = canvas_ref.clone();
            spawn_local(async move {
                if let Some(canvas) = canvas_ref_clone.cast::<HtmlCanvasElement>() {
                    let window = web_sys::window().expect("no window");
                    let dpr = window.device_pixel_ratio();
                    let rect = canvas.get_bounding_client_rect();
                    let width = (rect.width() * dpr) as u32;
                    let height = (rect.height() * dpr) as u32;

                    // Set initial canvas size
                    canvas.set_width(width);
                    canvas.set_height(height);

                    match WgpuRenderer::new_web(canvas.clone(), width, height).await {
                        Ok(renderer) => {
                            RENDERER.with(|r| {
                                *r.borrow_mut() = Some(renderer);
                            });
                            renderer_ready.set(true);

                            // Start animation loop after renderer is ready
                            start_animation_loop(canvas_ref_clone);
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to create wgpu renderer: {:?}", e).into(),
                            );
                        }
                    }
                }
            });

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
        // Get canvas
        if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
            let window = web_sys::window().expect("no window");

            // Resize canvas to match display size
            let dpr = window.device_pixel_ratio();
            let rect = canvas.get_bounding_client_rect();
            let display_width = (rect.width() * dpr) as u32;
            let display_height = (rect.height() * dpr) as u32;

            let needs_resize =
                canvas.width() != display_width || canvas.height() != display_height;

            if needs_resize {
                canvas.set_width(display_width);
                canvas.set_height(display_height);

                // Resize renderer
                RENDERER.with(|r| {
                    if let Some(renderer) = r.borrow_mut().as_mut() {
                        renderer.resize(display_width, display_height);
                    }
                });
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

                // Render using wgpu
                RENDERER.with(|r| {
                    if let Some(renderer) = r.borrow_mut().as_mut() {
                        renderer.render(&state.draw_cmds);
                    }
                });
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
