/// implementation of GameMode trait which defines game state

use piston::input::Event;
use image::RgbaImage;
/// messages used to Mode transition
pub enum ModeMessage {
    Transit(String),
    None,
}

/// altena16 handles several UI modes
pub trait GameMode {
    fn get_buf(&self) -> &RgbaImage;
    fn handle_event(&mut self, e: Event) -> ModeMessage;
    fn name(&self) -> &str;
}

pub struct CustomMode {}
