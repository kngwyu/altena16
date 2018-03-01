use image::{ImageBuffer, Primitive, Rgba};
use euclid::{TypedPoint2D, TypedRect, TypedSize2D, TypedVector2D};
use num_traits::{ToPrimitive, Zero};
use rect_iter::{RectIter, RectRange, TupleGet, TupleGetMut, XyGet, XyGetMut};
use std::collections::HashMap;
use std::ops::Range;
use std::slice;
use std::cmp;
pub struct DotSpace;
pub const DOT_HEIGHT: u16 = 240;
pub const DOT_WIDTH: u16 = 320;

pub type DotPoint = TypedPoint2D<i16, DotSpace>;
pub type DotSize = TypedSize2D<i16, DotSpace>;
pub type DotRect = TypedRect<i16, DotSpace>;
pub type DotVector = TypedVector2D<i16, DotSpace>;

trait ToDotVec {
    fn to_dot_vec(&self) -> DotVector;
}

impl ToDotVec for DotPoint {
    fn to_dot_vec(&self) -> DotVector {
        self.to_vector().to_dot_vec()
    }
}

impl ToDotVec for DotVector {
    fn to_dot_vec(&self) -> DotVector {
        self.clone()
    }
}

pub struct TileSpace;
pub type TilePoint = TypedPoint2D<u8, TileSpace>;
pub const TILE_SIZE: usize = 16;

/// Slide ractangle(ract.origin += offset).
fn slide_rect<T: ToDotVec>(rect: DotRect, offset: T) -> DotRect {
    let offset = offset.to_dot_vec();
    macro_rules! new {
        ($i:ident) => (rect.origin.$i + offset.$i)
    }
    DotRect {
        origin: DotPoint::new(new!(x), new!(y)),
        size: rect.size,
    }
}

/// Return tile size(16 × 16)
fn tile_size() -> DotSize {
    DotSize::new(TILE_SIZE as i16, TILE_SIZE as i16)
}

/// RectIter for tile
fn tile_iter() -> RectIter<usize> {
    RectRange::new(0, 0, TILE_SIZE, TILE_SIZE)
        .unwrap()
        .into_iter()
}

/// Calculate intersection of tile and DotRect and return range
fn get_tile_range(rect: DotRect, tile_origin: DotPoint) -> Option<RectRange<i16>> {
    let tile_rect = DotRect::new(tile_origin, tile_size());
    let inter = tile_rect.intersection(&rect)?;
    Some(RectRange::from_rect(inter)?)
}

fn bbox_intersects(
    bbox_1: DotRect,
    bbox_2: DotRect,
    offset_1: DotPoint,
    offset_2: DotPoint,
) -> bool {
    let s1 = slide_rect(bbox_1, offset_1);
    let s2 = slide_rect(bbox_2, offset_2);
    s1.intersects(&s2)
}

fn bbox_intersection(
    bbox_1: DotRect,
    bbox_2: DotRect,
    offset_1: DotPoint,
    offset_2: DotPoint,
) -> Option<DotRect> {
    let s1 = slide_rect(bbox_1, offset_1);
    let s2 = slide_rect(bbox_2, offset_2);
    s1.intersection(&s2)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TileDir {
    LeftUp,
    RightUp,
    RightDown,
    LeftDown,
}

impl TileDir {
    pub fn variants() -> slice::Iter<'static, TileDir> {
        use self::TileDir::*;
        const VARIANTS: &'static [TileDir] = &[LeftUp, RightUp, RightDown, LeftDown];
        VARIANTS.into_iter()
    }
    pub fn to_vec(&self) -> DotVector {
        let vec2d = |x, y| DotVector::new(x, y);
        match *self {
            LeftUp => vec2d(0, 0),
            RightUp => vec2d(1, 0),
            RightDown => vec2d(1, 1),
            LeftDown => vec2d(0, 1),
        }
    }
}

/// Leaf for MeshTree
#[derive(Copy, Clone)]
pub struct MeshLeaf {
    /// Object Mesh scaled to tile size.
    inner: [u16; TILE_SIZE],
    /// Bounding Box of meshed object.
    /// Its origin is based on upper left corner of tile.
    bbox: DotRect,
}

impl MeshLeaf {
    fn collide_l(
        &self,
        other: &MeshLeaf,
        offset_s: DotPoint,
        offset_o: DotPoint,
    ) -> Option<DotRect> {
        let line_mask = |range: &Range<i16>| {
            let len = range.end - range.start;
            let mask = (1 << len) - 1;
            let shift = range.start;
            move |b: u16| (b >> shift) & mask
        };
        let intersect = bbox_intersection(self.bbox, other.bbox, offset_s, offset_o)?;
        let tile_s = get_tile_range(intersect, offset_s)?;
        let tile_o = get_tile_range(intersect, offset_o)?;
        let mask_s = line_mask(&tile_s.x());
        let mask_o = line_mask(&tile_o.x());
        for (y_s, y_o) in tile_s.y().zip(tile_o.y()) {
            let masked_s = mask_s(self.inner[y_s as usize]);
            let masked_o = mask_o(other.inner[y_o as usize]);
            if (masked_s & masked_o) != 0 {
                return Some(intersect);
            }
        }
        None
    }
    fn collide_n(
        &self,
        other: &MeshNode,
        offset_s: DotPoint,
        offset_o: DotPoint,
    ) -> Option<DotRect> {
        if !bbox_intersects(self.bbox, other.bbox, offset_s, offset_o) {
            return None;
        }
        let compensate = |v: TileDir| v.to_vec() * other.scale;
        for &(ref child_o, dir) in &other.inner {
            let offset_o = offset_o + compensate(dir);
            match *child_o {
                MeshTree::Leaf(ref leaf) => {
                    if !bbox_intersects(self.bbox, leaf.bbox, offset_s, offset_o) {
                        continue;
                    }
                    let res = self.collide_l(leaf, offset_s, offset_o);
                    if res.is_some() {
                        return res;
                    }
                }
                MeshTree::Node(ref node) => {
                    if !bbox_intersects(self.bbox, node.bbox, offset_s, offset_o) {
                        continue;
                    }
                    let res = self.collide_n(node, offset_s, offset_o);
                    if res.is_some() {
                        return res;
                    }
                }
            }
        }
        None
    }
    fn from_buf(buf: &ImgBuf, range: RectRange<u32>) {}
}

/// Node for MeshTree
pub struct MeshNode {
    inner: Vec<(MeshTree, TileDir)>,
    bbox: DotRect,
    scale: i16,
}

impl MeshNode {
    fn collide(&self, other: &MeshNode, offset_s: DotPoint, offset_o: DotPoint) -> Option<DotRect> {
        if !bbox_intersects(self.bbox, other.bbox, offset_s, offset_o) {
            return None;
        }
        let compensate = |v: TileDir| v.to_vec() * self.scale;
        for &(ref child_s, dir) in &self.inner {
            let offset_s = offset_s + compensate(dir);
            match *child_s {
                MeshTree::Leaf(ref leaf) => {
                    if !bbox_intersects(leaf.bbox, other.bbox, offset_s, offset_o) {
                        continue;
                    }
                    let res = leaf.collide_n(other, offset_s, offset_o);
                    if res.is_some() {
                        return res;
                    }
                }
                MeshTree::Node(ref node) => {
                    if !bbox_intersects(node.bbox, other.bbox, offset_s, offset_o) {
                        continue;
                    }
                    let res = node.collide(other, offset_s, offset_o);
                    if res.is_some() {
                        return res;
                    }
                }
            }
        }
        None
    }
}

/// Utility type for collision detection
pub enum MeshTree {
    Leaf(MeshLeaf),
    Node(MeshNode),
}

impl MeshTree {
    /// Detect Collision
    fn collide(&self, other: &MeshTree, offset_s: DotPoint, offset_o: DotPoint) -> Option<DotRect> {
        match *self {
            MeshTree::Leaf(ref leaf_s) => match *other {
                MeshTree::Leaf(ref leaf_o) => leaf_s.collide_l(leaf_o, offset_s, offset_o),
                MeshTree::Node(ref node_o) => leaf_s.collide_n(node_o, offset_s, offset_o),
            },
            MeshTree::Node(ref node_s) => match *other {
                MeshTree::Leaf(ref leaf_o) => leaf_o.collide_n(node_s, offset_o, offset_s),
                MeshTree::Node(ref node_o) => node_s.collide(node_o, offset_s, offset_o),
            },
        }
    }
    fn from_buf_(buf: &ImgBuf, range: RectRange<u32>) {
        let (xlen, ylen) = (range.xlen(), range.ylen());
        if xlen <= TILE_SIZE as u32 && ylen <= TILE_SIZE as u32 {
            MeshLeaf::from_buf(buf, range);
            return;
        }
    }
    /// construct mesh from Image Buffer
    fn from_buf(buf: &ImgBuf) {
        let (h, w) = (buf.height(), buf.width());
    }
}

pub trait Collide {
    /// LeftUp Corner of Object
    fn origin(&self) -> DotPoint;
    fn mesh(&self) -> &MeshTree;
    fn collide(&self, other: &impl Collide) {
        let origin_s = self.origin();
        let origin_o = other.origin();
        self.mesh().collide(other.mesh(), origin_s, origin_o);
    }
}

/// altena don't support alpha blending, so just rgb is enough
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn from_rgba<T: Primitive>(rgba: &Rgba<T>) -> Option<Color> {
        if is_trans(rgba) {
            return None;
        }
        Some(Color {
            r: rgba[0].to_u8()?,
            g: rgba[1].to_u8()?,
            b: rgba[2].to_u8()?,
        })
    }
}

fn is_trans<T: Primitive>(rgba: &Rgba<T>) -> bool {
    rgba[3] == T::zero()
}

pub type Dot = Option<Color>;

/// 16×16 tile used to draw objects.
#[derive(Clone)]
pub struct Tile {
    /// Buffer of tile data
    inner: [Dot; TILE_SIZE * TILE_SIZE],
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            inner: [None; TILE_SIZE * TILE_SIZE],
        }
    }
}

impl XyGet for Tile {
    type Item = Dot;
    fn xy_get<T: ToPrimitive>(&self, x: T, y: T) -> Option<&Dot> {
        let (x, y) = (x.to_usize()?, y.to_usize()?);
        if TILE_SIZE <= x || TILE_SIZE <= y {
            return None;
        }
        Some(&self.inner[y * TILE_SIZE + x])
    }
}

impl XyGetMut for Tile {
    type Item = Dot;
    fn xy_get_mut<T: ToPrimitive>(&mut self, x: T, y: T) -> Option<&mut Dot> {
        let (x, y) = (x.to_usize()?, y.to_usize()?);
        if TILE_SIZE <= x || TILE_SIZE <= y {
            return None;
        }
        Some(&mut self.inner[y * TILE_SIZE + x])
    }
}

impl TupleGet for Tile {}
impl TupleGetMut for Tile {}

pub type ImgBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// 1 Frame of sprite
pub struct Frame {
    /// for drawing
    tiles: Vec<(Tile, TilePoint)>,
    /// for collision
    mesh: MeshTree,
    tree_base: TilePoint,
}

impl Frame {
    fn from_buf(buf: &ImgBuf) -> Option<()> {
        let (h, w) = (buf.height() as usize, buf.width() as usize);
        let tile_num = |len| (len + TILE_SIZE - 1) / TILE_SIZE;
        let tiles: Vec<_> = RectRange::zero_start(tile_num(w), tile_num(w))?
            .slide((1, 1))
            .into_iter()
            .map(|(tile_x, tile_y)| {
                let start = |t| TILE_SIZE * (t - 1);
                let (sx, sy) = (start(tile_x), start(tile_y));
                let end = |start, len| cmp::min(start + TILE_SIZE, len);
                let buf_rect = RectRange::new(sx, sy, end(sx, w), end(sy, h))
                    .expect("Invalid RectRange construnction in Frame::from_buf!");
                let tile = tile_iter().zip(buf_rect).fold(
                    Tile::default(),
                    |mut tile, (tile_t, (buf_x, buf_y))| {
                        let p = buf.get_pixel(buf_x as u32, buf_y as u32);
                        let dot = Color::from_rgba(p);
                        *tile.tuple_get_mut(tile_t)
                            .expect("Index Bug in Frame::from_buf!!") = dot;
                        tile
                    },
                );
                let tile_p = TilePoint::new(tile_x as u8 - 1, tile_y as u8 - 1);
                (tile, tile_p)
            })
            .collect();
        None
    }
}

/// Sprite
pub struct Sprite {
    frames: Vec<Frame>,
    frame_id_map: HashMap<String, usize>,
}

pub trait Drawable {}

#[cfg(test)]
mod screen_test {}
