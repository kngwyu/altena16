//! Sprite and scene management

use std::collections::HashMap;
use frame::Frame;
use frame::dottypes::*;

/// Sprite is a set of frames which an object has
pub struct Sprite {
    /// id for sprite
    name: String,
    /// node type of sprite
    ty: NodeType,
    childlen: HashMap<String, Sprite>,
}

enum NodeType {
    /// Root Sprite
    Root,
    /// child has relative position in parent's coordinate
    Child(DotPoint),
}
/// helper struct to define sprite hierarchy
pub struct SpriteBuilder {}

pub enum SpriteAction {

}

struct Timer {
    rest: u32,
}
