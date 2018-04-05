/// implementation of GameMode trait which defines game state

use piston::input::Event;
use image::RgbaImage;
use opengl_graphics::Texture;
use std::collections::{HashMap, hash_map::*};
/// messages used to Mode transition
pub enum AppMessage {
    Transit(String),
    None,
}

/// altena16 handles several application
pub trait App {
    fn get_buf(&self) -> Option<&RgbaImage>;
    fn draw_ui(&self);
    fn handle_event(&mut self, e: Event) -> AppMessage;
    fn name(&self) -> &str;
}

pub struct CustomMode {}
