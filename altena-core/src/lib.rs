#![feature(conservative_impl_trait, dyn_trait, iterator_try_fold, match_default_bindings, nll,
           try_trait, universal_impl_trait)]

extern crate ansi_term;
extern crate euclid;
extern crate graphics;
extern crate image;
#[macro_use]
extern crate log;
extern crate num_traits;
extern crate opengl_graphics;
extern crate piston;
extern crate rect_iter;
extern crate rusttype;
extern crate sdl2_window;
extern crate tuple_map;

mod app;
mod font;
mod frame;
mod input;
mod scene;
mod schedule;
mod simulator;
#[cfg(test)]
mod testutils;
mod tile;
mod ui;

use opengl_graphics::{Filter, GlGraphics, OpenGL, Texture, TextureSettings};
use sdl2_window::Sdl2Window;
use piston::window::WindowSettings;
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{Event, Input, Loop};
use tuple_map::*;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use app::{App, AppMessage};
use frame::dottypes::*;
use input::InputHandler;

/// clock counter type
/// currently we use Update event as a counter, but it may be changed in the future
pub type Clock = u64;

/// Inclusive time span
/// We use our own type instead of Range, to get 'Copy'
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: Clock,
    pub end: Clock,
}

impl Span {
    fn new(s: Clock, e: Clock) -> Span {
        Span { start: s, end: e }
    }
    fn length(&self) -> Clock {
        1 + self.end - self.start
    }
}

/// All game data in altena16
pub struct AltenaCore {
    /// OpenGL context
    gl: GlGraphics,
    apps: HashMap<String, Box<dyn App>>,
    current_app: String,
    /// OpenGL Texture
    /// We use only .update method to draw on screen
    texture: Cell<Texture>,
    /// game ended or not
    pub end: bool,
    /// REAL_SCREEN_SIZE / DOT_SCREEN_SIZE
    x_scale: f64,
    y_scale: f64,
    upd_count: Clock,
    input_handle: InputHandler,
}

impl AltenaCore {
    fn get_scale(w: u32, h: u32) -> (f64, f64) {
        let scale = |(d, s)| f64::from(s) / f64::from(d);
        ((DOT_WIDTH, w), (DOT_HEIGHT, h)).map(scale)
    }

    fn register_app(app: impl App) {}

    fn from_setting(setting: AltenaSetting) -> AltenaCore {
        let texture_setting = TextureSettings::new().filter(Filter::Nearest);
        let texture = Texture::empty(&texture_setting).expect("couldn't make OpenGL texture");
        let (sw, sh) = Self::get_scale(setting.width, setting.height);
        AltenaCore {
            gl: GlGraphics::new(setting.opengl),
            apps: HashMap::new(),
            current_app: "".to_owned(),
            texture: Cell::new(texture),
            end: false,
            x_scale: sw,
            y_scale: sh,
            upd_count: 0,
            input_handle: InputHandler::default(),
        }
    }
    fn handle_events(&mut self, event: Event) {
        match event {
            Event::Input(input) => {
                if let Input::Resize(w, h) = input {
                    let (x, y) = Self::get_scale(w, h);
                    self.x_scale = x;
                    self.y_scale = y;
                } else {
                    self.input_handle.handle(input, self.upd_count);
                }
            }
            Event::Loop(loop_event) => {
                let mut app = match self.apps.get_mut(&self.current_app) {
                    Some(s) => s,
                    None => {
                        warn!("no app named {}", self.current_app);
                        return;
                    }
                };
                match loop_event {
                    Loop::Render(args) => {
                        if let Some(buf) = app.get_buf() {
                            let mut t = self.texture.get_mut();
                            t.update(&buf);
                            // TODO: custom transform matrix support
                            let (xs, ys) = (self.x_scale, self.y_scale);
                            self.gl.draw(args.viewport(), |ctx, gl| {
                                use graphics::*;
                                // TODO: custom clear color support
                                clear([1.0; 4], gl);
                                let trans = ctx.transform.scale(xs, ys);
                                image(t, trans, gl);
                            });
                        }
                    }
                    Loop::Update(args) => {
                        self.upd_count += 1;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// application setting(builder)
pub struct AltenaSetting {
    width: u32,
    height: u32,
    max_fps: u64,
    ups: u64,
    opengl: OpenGL,
    resizable: bool,
}

impl AltenaSetting {
    const DEFAULT_MAX_FPS: u64 = 30;
    const DEFAULT_UPS: u64 = 60;
    const DEFAULT_WIDTH: u32 = 640;
    const DEFAULT_HEIGHT: u32 = 480;
    pub fn new() -> AltenaSetting {
        AltenaSetting {
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
            max_fps: Self::DEFAULT_MAX_FPS,
            ups: Self::DEFAULT_UPS,
            opengl: OpenGL::V2_1,
            resizable: false,
        }
    }
    pub fn width(&mut self, width: u32) -> &mut AltenaSetting {
        self.width = width;
        self
    }
    pub fn height(&mut self, height: u32) -> &mut AltenaSetting {
        self.height = height;
        self
    }
    pub fn max_fps(&mut self, f: u64) -> &mut AltenaSetting {
        self.max_fps = f;
        self
    }
    pub fn ups(&mut self, u: u64) -> &mut AltenaSetting {
        self.ups = u;
        self
    }
    pub fn opengl(&mut self, gl: OpenGL) -> &mut AltenaSetting {
        self.opengl = gl;
        self
    }
    pub fn resizable(&mut self, b: bool) -> &mut AltenaSetting {
        self.resizable = b;
        self
    }
}

/// This function defines altena's main loop.
///
/// # Example
/// ```ignore
/// fn main() {
///     let setting = AltenaSetting::new();
///     let (main_loop, altena) = altena_core::main_loop(setting);
///     let mut end = false;
///     while !end {
///         main_loop();
///         end = altena.borrow().end;
///     }
/// }
/// ```
pub fn main_loop(setting: AltenaSetting) -> (impl FnMut(), Rc<RefCell<AltenaCore>>) {
    let opengl = setting.opengl;
    let mut window: Sdl2Window = WindowSettings::new("SDL Window", (setting.width, setting.height))
        .opengl(opengl)
        .exit_on_esc(true) // it's useful for debug
        .srgb(false)
        .vsync(true)
        .resizable(setting.resizable)
        .build()
        .expect("Failed to build window!");
    let event_setting = EventSettings::new()
        .max_fps(setting.max_fps)
        .ups(setting.ups);
    let mut events = Events::new(event_setting);
    let altena = Rc::new(RefCell::new(AltenaCore::from_setting(setting)));
    (
        {
            let altena = Rc::clone(&altena);
            move || {
                if let Some(event) = events.next(&mut window) {
                    let mut altena = altena.borrow_mut();
                    altena.handle_events(event);
                } else {
                    let mut altena = altena.borrow_mut();
                    altena.end = true;
                }
            }
        },
        altena,
    )
}
