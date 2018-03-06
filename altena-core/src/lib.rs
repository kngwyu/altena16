#![feature(conservative_impl_trait, dyn_trait, iterator_try_fold, match_default_bindings, nll,
           universal_impl_trait)]

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
extern crate sdl2_window;

mod errors;
mod screen;
mod world;
mod mode;
#[cfg(test)]
mod testutils;

use opengl_graphics::{Filter, GlGraphics, OpenGL, Texture, TextureSettings};
use sdl2_window::Sdl2Window;
use piston::window::WindowSettings;
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{Event, RenderArgs, RenderEvent};
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::option::Option;
use std::collections::HashMap;

use mode::{GameMode, ModeMessage};

/// Game Data
pub struct AltenaCore {
    gl: GlGraphics,
    states: HashMap<String, Box<dyn GameMode>>,
    current_state: String,
    texture: Cell<Texture>,
    pub end: bool,
}

impl AltenaCore {
    fn from_setting(setting: AltenaSetting) -> AltenaCore {
        let texture_setting = TextureSettings::new().filter(Filter::Nearest);
        let texture = Texture::empty(&texture_setting).expect("couldn't make OpenGL texture");
        AltenaCore {
            gl: GlGraphics::new(setting.opengl),
            states: HashMap::new(),
            current_state: "".to_owned(),
            texture: Cell::new(texture),
            end: false,
        }
    }
    fn handle_events(&mut self, e: Event) {
        let mut state = match self.states.get_mut(&self.current_state) {
            Some(s) => s,
            None => {
                warn!("no state named {}", self.current_state);
                return;
            }
        };
        if let Some(args) = e.render_args() {
            let buf = state.get_buf();
            let mut t = self.texture.get_mut();
            t.update(&buf);
            // TODO: custom transform matrix support
            self.gl.draw(args.viewport(), |ctx, gl| {
                use graphics::*;
                clear([1.0; 4], gl);
                let trans = ctx.transform.scale(2.0, 2.0);
                graphics::image(t, trans, gl);
            });
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
}

impl AltenaSetting {
    const DEFAULT_MAX_FPS: u64 = 30;
    const DEFAULT_UPS: u64 = 50;
    pub fn new(width: u32, height: u32) -> AltenaSetting {
        AltenaSetting {
            width: width,
            height: height,
            max_fps: Self::DEFAULT_MAX_FPS,
            ups: Self::DEFAULT_UPS,
            opengl: OpenGL::V2_1,
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
}

pub fn main_loop(setting: AltenaSetting) -> (impl FnMut(), Rc<RefCell<AltenaCore>>) {
    let opengl = setting.opengl;
    let mut window: Sdl2Window = WindowSettings::new("SDL Window", (setting.width, setting.height))
        .opengl(opengl)
        .exit_on_esc(true)
        .srgb(false)
        .vsync(true)
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
