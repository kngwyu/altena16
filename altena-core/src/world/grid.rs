use screen::{DotPoint, DotRect, DotSize, DotSpace, Tile, DOT_HEIGHT, DOT_WIDTH};
use euclid;
pub struct GridSpace;
type GridPoint = euclid::TypedPoint2D<u16, GridSpace>;
type GridVector = euclid::TypedVector2D<u16, GridSpace>;
type GridSize = euclid::TypedSize2D<u16, GridSpace>;
type GridRect = euclid::TypedRect<u16, GridSpace>;

/// Manager
pub struct GridWrold {}

/// Map
struct GridMap {
    map: Vec<Vec<Tile>>,
    draw_range: DotRect,
}

#[derive(Clone, Debug)]
pub struct ObjectEntity {}
