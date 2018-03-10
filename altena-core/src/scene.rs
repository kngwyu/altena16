//! Sprite and scene management

use std::collections::HashMap;
use frame::Frame;
use frame::dottypes::*;

/// Sprite
pub struct Sprite {
    /// id for sprite
    name: String,
    frames: HashMap<String, Frame>,
}
