//! Sprite and scene management

use std::collections::HashMap;
use frame::Frame;
use frame::dottypes::*;

/// Sprite is a set of `Drawing objects` which an object has
pub struct Sprite {
    /// id for sprite
    name: String,
    /// node type
    typ: NodeType,
    /// sprites with coordinate related to its parent
    childlen: HashMap<String, Sprite>,
    /// draw priority
    priority: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NodeType {
    /// Window
    Root,
    /// child has relative position in parent's coordinate
    Child(DotPoint),
}

/// Action to Sprite
pub enum SpriteAction {
    Move(DotVector),
    ChangeFrame(String),
    Rotate(u16),
    Scale(u8),
}
