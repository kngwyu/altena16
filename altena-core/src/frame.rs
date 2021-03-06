//! tile, frame, collision detection

use euclid::{point2, rect, vec2, TypedRect, TypedVector2D};
use image::{Rgba, RgbaImage};
use num_traits::{Num, ToPrimitive};
use rect_iter::{copy_rect, copy_rect_conv, gen_rect_conv, Get2D, IntoTuple2, RectRange};
use tuple_map::TupleMap2;

use std::cmp::{max, min};
use std::fmt;
use std::ops::Range;
use std::slice;

use tile::tiletypes::*;
use tile::{Alpha, AltenaAlpha, Color, Dot, Tile};

pub mod dottypes {
    use euclid::*;
    pub struct DotSpace;
    pub const DOT_HEIGHT: u16 = 240;
    pub const DOT_WIDTH: u16 = 320;
    pub type DotPoint = TypedPoint2D<i16, DotSpace>;
    pub type DotSize = TypedSize2D<i16, DotSpace>;
    pub type DotRect = TypedRect<i16, DotSpace>;
    pub type DotVector = TypedVector2D<i16, DotSpace>;
}

use self::dottypes::*;
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

/// Slide ractangle(ract.origin += offset).
fn slide_rect<T: Num + Copy, P: IntoTuple2<T>, Unit>(
    rect: TypedRect<T, Unit>,
    offset: P,
) -> TypedRect<T, Unit> {
    let offset = offset.into_tuple2();
    TypedRect {
        origin: point2(rect.origin.x + offset.0, rect.origin.y + offset.1),
        size: rect.size,
    }
}

/// Return tile size(16 × 16)
fn tile_size() -> DotSize {
    DotSize::new(TILE_SIZE as i16, TILE_SIZE as i16)
}

fn tile_num(len: usize) -> usize {
    (len + TILE_SIZE - 1) / TILE_SIZE
}

/// Calculate intersection of tile and DotRect and return range
fn get_tile_range(rect: DotRect, offset: DotPoint) -> Option<RectRange<i16>> {
    let tile_rect = DotRect::new(point2(0, 0), tile_size());
    let rect = slide_rect(rect, offset.to_vector() * -1);
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
    pub fn to_vec<T: From<u8>, U>(&self) -> TypedVector2D<T, U> {
        use self::TileDir::*;
        let ret = |x, y| vec2(T::from(x), T::from(y));
        match self {
            LeftUp => ret(0, 0),
            RightUp => ret(1, 0),
            RightDown => ret(1, 1),
            LeftDown => ret(0, 1),
        }
    }
    pub fn x<T: From<u8>>(&self) -> T {
        use self::TileDir::*;
        match self {
            LeftUp => T::from(0u8),
            RightUp => T::from(1u8),
            RightDown => T::from(1u8),
            LeftDown => T::from(0u8),
        }
    }
    pub fn y<T: From<u8>>(&self) -> T {
        use self::TileDir::*;
        match self {
            LeftUp => T::from(0u8),
            RightUp => T::from(0u8),
            RightDown => T::from(1u8),
            LeftDown => T::from(1u8),
        }
    }
}

/// Leaf for MeshTree
#[derive(Copy, Clone)]
pub struct MeshLeaf {
    /// Object Mesh scaled to tile size.
    inner: [u64; TILE_SIZE],
    /// Bounding Box of meshed object.
    /// Its origin is based on upper left corner of tile.
    bbox: DotRect,
}

impl fmt::Debug for MeshLeaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "mesh: {{")?;
        for i in 0..TILE_SIZE {
            writeln!(f, "{:064b}", self.inner[i])?;
        }
        writeln!(f, "}}, bbox: {:?}", self.bbox)
    }
}

impl MeshLeaf {
    fn collide_l(
        &self,
        other: &MeshLeaf,
        offset_s: DotPoint,
        offset_o: DotPoint,
    ) -> Option<DotRect> {
        let line_mask = |range: &Range<i16>| {
            let len = (range.end - range.start) as usize;
            let mask = u64::max_value() << ((TILE_SIZE - len) * 4);
            let shift = range.start * 4;
            move |b: u64| (b << shift) & mask
        };
        let intersect = bbox_intersection(self.bbox, other.bbox, offset_s, offset_o)?;
        let range_s = get_tile_range(intersect, offset_s)?;
        let range_o = get_tile_range(intersect, offset_o)?;
        let (mask_s, mask_o) = (range_s.get_x(), range_o.get_x()).map(line_mask);
        range_s
            .cloned_y()
            .zip(range_o.cloned_y())
            .find(|&(y_s, y_o)| {
                let masked_s = mask_s(self.inner[y_s as usize]);
                let masked_o = mask_o(other.inner[y_o as usize]);
                (masked_s & masked_o) != 0
            })
            .map(|_| intersect)
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
        let compensate = |v: &TileDir| v.to_vec() * other.scale * (TILE_SIZE / 2) as i16;
        other
            .inner
            .iter()
            .filter_map(|(child, dir)| {
                let offset_o = offset_o + compensate(dir);
                match child {
                    MeshTree::Leaf(leaf) => {
                        if !bbox_intersects(self.bbox, leaf.bbox, offset_s, offset_o) {
                            return None;
                        }
                        self.collide_l(leaf, offset_s, offset_o)
                    }
                    MeshTree::Node(node) => {
                        if !bbox_intersects(self.bbox, node.bbox, offset_s, offset_o) {
                            return None;
                        }
                        self.collide_n(node, offset_s, offset_o)
                    }
                }
            })
            .nth(0)
    }
    fn from_buf(buf: &RgbaImage, range: RectRange<u32>) -> Option<MeshLeaf> {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (TILE_SIZE, TILE_SIZE, 0, 0);
        let mut upd_minmax = |x, y| {
            min_x = min(min_x, x);
            min_y = min(min_y, y);
            max_x = max(max_x, x);
            max_y = max(max_y, y);
        };
        let inner = tile_rect().into_iter().zip(range).fold(
            [0u64; TILE_SIZE],
            |mut array, ((x, y), (buf_x, buf_y))| {
                let p = buf.get_pixel(buf_x, buf_y);
                let collision_bits = p.collision_bits();
                if collision_bits != 0 {
                    upd_minmax(x, y);
                    let shift = (TILE_SIZE - x - 1) * 4;
                    array[y] |= u64::from(collision_bits) << shift;
                }
                array
            },
        );
        Some(MeshLeaf {
            inner: inner,
            bbox: RectRange::from_corners((min_x, min_y), (max_x + 1, max_y + 1))?
                .to_i16()?
                .to_rect(),
        })
    }
    fn get_debug_buf(&self) -> Vec<Vec<u64>> {
        self.inner.iter().map(|&u| vec![u]).collect()
    }
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
        let compensate = |v: &TileDir| v.to_vec() * self.scale * TILE_SIZE as i16;
        self.inner
            .iter()
            .filter_map(|(child_s, dir)| {
                let offset_s = offset_s + compensate(dir);
                match child_s {
                    MeshTree::Leaf(leaf) => {
                        if !bbox_intersects(leaf.bbox, other.bbox, offset_s, offset_o) {
                            return None;
                        }
                        leaf.collide_n(other, offset_s, offset_o)
                    }
                    MeshTree::Node(node) => {
                        if !bbox_intersects(node.bbox, other.bbox, offset_s, offset_o) {
                            return None;
                        }
                        node.collide(other, offset_s, offset_o)
                    }
                }
            })
            .nth(0)
    }
    fn get_debug_buf(&self) -> Vec<Vec<u64>> {
        let uscale = self.scale as usize;
        let child_scale = uscale / 2;
        self.inner.iter().fold(
            vec![vec![0u64; uscale]; TILE_SIZE * uscale],
            |mut vec, (child, dir)| {
                let child_buf = match child {
                    MeshTree::Leaf(leaf) => leaf.get_debug_buf(),
                    MeshTree::Node(node) => node.get_debug_buf(),
                };
                let res_range = RectRange::zero_start(1, TILE_SIZE)
                    .unwrap()
                    .scale(child_scale)
                    .slide((
                        child_scale * dir.x::<usize>(),
                        child_scale * dir.y::<usize>() * TILE_SIZE,
                    ));
                let buf_range = RectRange::zero_start(child_buf[0].len(), child_buf.len()).unwrap();
                copy_rect(&child_buf, &mut vec, buf_range, res_range).unwrap();
                vec
            },
        )
    }
    #[allow(dead_code)]
    fn print_leaf(&self) {
        for (child_s, dir) in &self.inner {
            match child_s {
                MeshTree::Leaf(leaf) => println!("dir: {:?}, leaf: {:?}", dir, leaf),
                MeshTree::Node(node) => node.print_leaf(),
            }
        }
    }
}

impl fmt::Debug for MeshNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "mesh: {{")?;
        let buf = self.get_debug_buf();
        for i in 0..buf.len() {
            for j in 0..buf[i].len() {
                write!(f, "{:064b}", buf[i][j])?;
            }
            writeln!(f, "")?;
        }
        writeln!(f, "}}, bbox: {:?}", self.bbox)
    }
}

/// Utility type for collision detection
#[derive(Debug)]
pub enum MeshTree {
    Leaf(MeshLeaf),
    Node(MeshNode),
}

impl MeshTree {
    /// Detect Collision
    fn collide(&self, other: &MeshTree, offset_s: DotPoint, offset_o: DotPoint) -> Option<DotRect> {
        match self {
            MeshTree::Leaf(leaf_s) => match other {
                MeshTree::Leaf(leaf_o) => leaf_s.collide_l(leaf_o, offset_s, offset_o),
                MeshTree::Node(node_o) => leaf_s.collide_n(node_o, offset_s, offset_o),
            },
            MeshTree::Node(node_s) => match other {
                MeshTree::Leaf(leaf_o) => leaf_o.collide_n(node_s, offset_o, offset_s),
                MeshTree::Node(node_o) => node_s.collide(node_o, offset_s, offset_o),
            },
        }
    }
    fn from_buf_(buf: &RgbaImage, range_orig: RectRange<u32>) -> Option<MeshTree> {
        let get_scale = |max_len: u32| {
            let mut len = TILE_SIZE as u32;
            for scale in 1..6 {
                if max_len <= len {
                    return scale;
                }
                len *= 2;
            }
            panic!("Mesh size {} is too big and not supported!", len)
        };
        let (xlen, ylen) = (range_orig.xlen(), range_orig.ylen());
        let scale = get_scale(max(xlen, ylen));
        if scale == 1 {
            let leaf = MeshLeaf::from_buf(buf, range_orig)?;
            return Some(MeshTree::Leaf(leaf));
        }
        let mut bbox_res: Option<DotRect> = None;
        let child_scale = scale / 2;
        let children = TileDir::variants()
            .filter_map(|dir| {
                let left_up: TypedVector2D<_, DotSpace> =
                    dir.to_vec() * child_scale * TILE_SIZE as u32;
                let right_down = left_up + vec2(1, 1) * child_scale * TILE_SIZE as u32;
                let divided = RectRange::from_corners(left_up, right_down).unwrap();
                let inter = range_orig.intersection(&divided)?;
                let res = MeshTree::from_buf_(buf, inter)?;
                let bbox = match res {
                    MeshTree::Leaf(ref leaf) => leaf.bbox,
                    MeshTree::Node(ref node) => node.bbox,
                };
                let bbox = slide_rect(bbox, (left_up.x as i16, left_up.y as i16));
                bbox_res = match bbox_res {
                    Some(b) => Some(b.union(&bbox)),
                    None => Some(bbox),
                };
                Some((res, *dir))
            })
            .collect();
        let res = MeshNode {
            inner: children,
            bbox: bbox_res?,
            scale: scale as i16,
        };
        Some(MeshTree::Node(res))
    }
    /// construct mesh from Image Buffer
    pub fn from_buf(buf: &RgbaImage) -> Option<MeshTree> {
        let (h, w) = (buf.height(), buf.width());
        let range = RectRange::zero_start(w, h).unwrap();
        Self::from_buf_(buf, range)
    }

    fn get_debug_buf(&self) -> Vec<Vec<u64>> {
        match self {
            MeshTree::Node(node) => node.get_debug_buf(),
            MeshTree::Leaf(leaf) => leaf.get_debug_buf(),
        }
    }

    fn bbox(&self) -> DotRect {
        match self {
            MeshTree::Node(node) => node.bbox,
            MeshTree::Leaf(leaf) => leaf.bbox,
        }
    }
}

/// 1 Frame of sprite
pub struct Frame {
    /// name of Frame
    name: String,
    /// collision
    mesh: MeshTree,
    /// for drawing
    tiles: Vec<(Tile, TilePoint)>,
    alpha: Alpha,
    /// to restore png image
    w_orig: usize,
    h_orig: usize,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Frame name: {}", self.name)?;
        writeln!(f, "tiles: {{")?;
        let buf = self.get_debug_buf().unwrap();
        for i in 0..buf.len() {
            for j in 0..buf[0].len() {
                let dot = buf.get_xy(j, i).unwrap();
                write!(f, "{:?}", dot)?;
            }
            writeln!(f, "")?;
        }
        writeln!(f, "}}")
    }
}

impl Frame {
    pub fn from_buf(buf: &RgbaImage, name: &str) -> Option<Frame> {
        let (h, w) = (buf.height(), buf.width()).map(|u| u as usize);
        let tiles: Vec<_> = RectRange::zero_start(tile_num(w), tile_num(h))?
            .slide((1, 1))
            .into_iter()
            .map(|(tile_x, tile_y)| {
                let (sx, sy) = (tile_x, tile_y).map(|t| TILE_SIZE * (t - 1));
                let (ex, ey) = ((sx, w), (sy, h)).map(|(start, len)| min(start + TILE_SIZE, len));
                let buf_rect = RectRange::new(sx, sy, ex, ey).unwrap();
                let tile =
                    gen_rect_conv(buf, Tile::default, buf_rect, tile_rect(), Color::from_rgba)
                        .expect("Index bug in Frame::frame_buf!!!");
                let tile_p = point2(tile_x as u8 - 1, tile_y as u8 - 1);
                (tile, tile_p)
            })
            .collect();
        let mesh = MeshTree::from_buf(buf)?;
        let alpha = buf.pixels()
            .fold(Alpha(0), |mut alpha, p| *alpha.max(p.alpha()));
        Some(Frame {
            name: name.to_owned(),
            mesh: mesh,
            tiles: tiles,
            alpha: alpha,
            w_orig: w,
            h_orig: h,
        })
    }

    fn get_debug_buf(&self) -> Option<Vec<Vec<Dot>>> {
        let (w, h) = (self.w_orig, self.h_orig);
        self.tiles.iter().try_fold(
            vec![vec![Dot::default(); w]; h],
            |mut buf, (tile, point)| {
                let (sx, sy) = (point.x, point.y).map(|t| TILE_SIZE * t as usize);
                let (ex, ey) = ((sx, w), (sy, h)).map(|(start, len)| min(start + TILE_SIZE, len));
                let buf_rect = RectRange::new(sx, sy, ex, ey)?;
                copy_rect(tile, &mut buf, tile_rect(), buf_rect).ok()?;
                Some(buf)
            },
        )
    }

    fn get_color_buf(&self, cnv: &impl Fn(&Dot) -> Rgba<u8>) -> Option<RgbaImage> {
        let (w, h) = (self.w_orig, self.h_orig);
        self.tiles.iter().try_fold(
            RgbaImage::new(w as u32, h as u32),
            |mut buf, (tile, point)| {
                let (sx, sy) = (point.x, point.y).map(|t| TILE_SIZE * t as usize);
                let (ex, ey) = ((sx, w), (sy, h)).map(|(start, len)| min(start + TILE_SIZE, len));
                let buf_rect = RectRange::new(sx, sy, ex, ey)?;
                copy_rect_conv(tile, &mut buf, tile_rect(), buf_rect, cnv).ok()?;
                Some(buf)
            },
        )
    }
    fn restore_buf(&self) -> Option<RgbaImage> {
        let color_buf = self.get_color_buf(&|d| match d {
            Some(rgb) => {
                let mut res = rgb.to_rgba();
                res[3] = self.alpha.0 << 4;
                res
            }
            None => {
                let mut res = Rgba { data: [255; 4] };
                res[3] = 0;
                res
            }
        })?;
        let mesh = self.mesh.get_debug_buf();
        let get_bits = |x: usize, y: usize| {
            let mo = x % TILE_SIZE;
            let x = x / TILE_SIZE;
            let shift = (TILE_SIZE - 1 - mo) * 4;
            (mesh[y][x] >> shift) as u8 & 0b00001111
        };
        let c_range = RectRange::zero_start(color_buf.width(), color_buf.height()).unwrap();
        Some(c_range.into_iter().fold(color_buf, |mut buf, (x, y)| {
            let bits = get_bits(x as usize, y as usize);
            let p = buf.get_pixel_mut(x, y);
            p[3] |= bits;
            buf
        }))
    }

    fn bbox(&self) -> DotRect {
        self.mesh.bbox()
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

#[cfg(test)]
mod frame_test {
    use super::*;
    use testutils::{load_frame, load_img, load_mesh};
    #[test]
    fn load_1tile() {
        let frame = load_frame("../test-assets/chara1.png");
        println!("{:?}", frame);
        assert_eq!(rect(2, 2, 11, 14), frame.bbox());
    }
    #[test]
    fn load_1pixel() {
        let frame = load_frame("../test-assets/bullet.png");
        println!("{:?}", frame);
        assert_eq!(rect(7, 8, 1, 1), frame.bbox());
    }
    #[test]
    fn load_4tile() {
        let frame = load_frame("../test-assets/chara2.png");
        println!("{:?}", frame);
        assert_eq!(rect(1, 2, 30, 30), frame.bbox());
    }
    #[test]
    fn collide_l_1() {
        let bullet = load_mesh("../test-assets/bullet.png");
        let chara1 = load_mesh("../test-assets/chara1.png");
        let c = chara1.collide(&bullet, point2(0, 0), point2(0, 0));
        assert_eq!(c, Some(rect(7, 8, 1, 1)))
    }
    #[test]
    fn collide_l_2() {
        let bullet = load_mesh("../test-assets/bullet.png");
        let chara1 = load_mesh("../test-assets/chara1.png");
        let c = chara1.collide(&bullet, point2(16, 16), point2(12, 11));
        assert_eq!(c, Some(rect(19, 19, 1, 1)));
    }
    #[test]
    fn collide_n_1() {
        let chara1 = load_mesh("../test-assets/chara1.png");
        let chara2 = load_mesh("../test-assets/chara2.png");
        let c = chara2.collide(&chara1, point2(0, 0), point2(19, 16));
        assert_eq!(c, Some(rect(21, 18, 10, 14)));
    }
    #[test]
    fn frame_to_img_buf() {
        let img = load_img("../test-assets/chara2.png");
        let chara2 = load_frame("../test-assets/chara2.png");
        let chara_img = chara2.restore_buf().unwrap();
        let range = RectRange::zero_start(img.width(), img.height()).unwrap();
        assert!(range.into_iter().all(|p| {
            let orig = *img.get_point(p).unwrap();
            let converted = *chara_img.get_point(p).unwrap();
            orig == converted
        }));
    }
}
