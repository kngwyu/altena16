use frame::{DotPoint, DotRect, DotSize, Tile, DOT_HEIGHT, DOT_WIDTH};

/// Object and Map manager
/// We use DotScale as unit scale in PlatformerWorld
pub struct World {}

/// Map
struct Map {
    /// static map data
    draw_map: Vec<Vec<Tile>>,
    /// drawing range
    draw_range: DotRect,
}

/// Object which can't move and transfor
struct StaticObject {
    /// upper left point
    base_p: DotPoint,
    color: Tile,
}

/// Object which can move
// 本当はspriteが必要だけどとりあえず省略
struct Player {
    base_p: DotPoint,
    color: Tile,
}
