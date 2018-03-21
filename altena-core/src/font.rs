use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::fmt;
use std::error::Error;
use rusttype::{point, Font, FontCollection, Point, Scale};

use rect_iter::{GetMut2D, IndexError, RectRange, ToPoint};
use tile::tiletypes::*;
use tile::{Alpha, Blend, Color};
use tuple_map::TupleMap2;

#[derive(Copy, Clone, Debug)]
pub enum FontError {
    NoFont(char),
    Index(IndexError),
}

impl Error for FontError {
    fn description(&self) -> &str {
        "Font Error"
    }
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            FontError::NoFont(c) => write!(f, "NoFont: {}", c),
            FontError::Index(id) => write!(f, "Index: {}", id),
        }
    }
}

pub struct FontHandle<'a> {
    font: Font<'a>,
    cache: HashMap<(char, u8), FontCache>,
}

impl<'a> FontHandle<'a> {
    pub fn new(font: &'a [u8]) -> Self {
        let collection = FontCollection::from_bytes(font);
        let font = collection.into_font().expect("Invalid font data");
        Self {
            font: font,
            cache: HashMap::new(),
        }
    }

    pub fn draw<B, I>(
        &mut self,
        buf: &mut B,
        c: char,
        setting: &FontSetting,
    ) -> Result<(), FontError>
    where
        B: GetMut2D<Item = I>,
        I: Blend,
    {
        let glyph = match self.font.glyph(c) {
            Some(g) => g,
            None => return Err(FontError::NoFont(c)),
        };
        let glyph = glyph.scaled(setting.scale);
        let glyph = glyph.positioned(setting.start);
        let mut res = Ok(());
        let query = (c, setting.scale_u);
        let scale_u = u32::from(setting.scale_u);
        let start_u = (setting.start.x, setting.start.y).map(|f| f as u32);
        match self.cache.entry(query) {
            Entry::Occupied(o_e) => {
                let ranges = (setting.start.x, setting.start.y).map(|u| {
                    let u = u as u32;
                    u..u + scale_u
                });
                let range = RectRange::from_ranges(ranges.0, ranges.1).unwrap();
                range.into_iter().for_each(|(x, y)| {
                    let alpha = o_e.get().get(x, y);
                    let p = (x, y).add(start_u);
                    match buf.get_mut_point_r(p) {
                        Ok(b) => b.blend(setting.color, alpha),
                        Err(id) => res = Err(FontError::Index(id)),
                    }
                });
            }
            Entry::Vacant(v_e) => {
                let mut cached_font = FontCache::new(scale_u as usize);
                if let Some(bbox) = glyph.pixel_bounding_box() {
                    let offset = (bbox.min.x, bbox.max.y).map(|i| i as u32);
                    glyph.draw(|x, y, v| {
                        let mut alpha = Alpha::from_f32(v);
                        // TODO: parameter fix
                        if alpha.0 < 5 {
                            return;
                        }
                        alpha.plus(3);
                        let p = (x, y).add(offset);
                        match buf.get_mut_point_r(p) {
                            Ok(b) => b.blend(setting.color, alpha),
                            Err(id) => res = Err(FontError::Index(id)),
                        }
                        let (x, y) = p.sub(start_u);
                        cached_font.set(x, y, alpha);
                    });
                }
                v_e.insert(cached_font);
            }
        }
        res
    }
}

#[derive(Clone, Debug)]
pub struct FontSetting {
    color: Color,
    scale: Scale,
    scale_u: u8,
    start: Point<f32>,
}

impl FontSetting {
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = c;
        self
    }
    pub fn new() -> Self {
        FontSetting {
            color: Color::black(),
            scale: Scale::uniform(TILE_SIZE as f32),
            scale_u: TILE_SIZE as u8,
            start: point(0.0, 0.0),
        }
    }
    pub fn scale(&mut self, s: u8) -> &mut Self {
        self.scale = Scale::uniform(f32::from(s));
        self.scale_u = s;
        self
    }
    pub fn start<P, T>(&mut self, p: P) -> &mut Self
    where
        P: ToPoint<T>,
        T: Into<f32>,
    {
        let (x, y) = p.to_point().map(T::into);
        self.start = point(x, y);
        self
    }
}

#[derive(Clone, Debug)]
struct FontCache {
    inner: Vec<u8>,
    scale: usize,
}

impl FontCache {
    fn new(scale: usize) -> FontCache {
        let len = (scale * scale + 1) / 2;
        FontCache {
            inner: vec![0u8; len],
            scale: scale,
        }
    }
    fn set(&mut self, x: u32, y: u32, val: Alpha) {
        let (x, y) = (x, y).map(|a| a as usize);
        let id = self.scale * y + x;
        let id2 = id / 2;
        if (id & 1) == 1 {
            self.inner[id2] |= val.0 << 4;
        } else {
            self.inner[id2] |= val.0;
        };
    }
    fn get(&self, x: u32, y: u32) -> Alpha {
        let (x, y) = (x, y).map(|a| a as usize);
        let pos = self.scale * y + x;
        let id = pos / 2;
        let val = if (pos & 1) == 1 {
            (self.inner[id] & 0b11110000) >> 4
        } else {
            self.inner[id] & 0b00001111
        };
        Alpha(val)
    }
}

mod font_test {
    use super::*;
    use tile::Tile;
    use rect_iter::Get2D;
    use test::Bencher;
    const MIGU: &[u8; 3137552] = include_bytes!("../../assets/migu-1m-regular.ttf");
    #[test]
    fn draw_tile() {
        let setting = FontSetting::new();
        let mut font = FontHandle::new(&MIGU[..]);
        let mut tile = Tile::new(Some(Color::white()));
        for c in "あいうえおかきくけこ隣の客はよく柿食う客だabcdefg".chars() {
            font.draw(&mut tile, c, &setting).unwrap();
            println!("{:?}", tile);
            tile = Tile::new(Some(Color::white()));
        }
    }

    #[test]
    fn draw_tile_cached() {
        let setting = FontSetting::new();
        let mut font = FontHandle::new(&MIGU[..]);
        let mut tile = Tile::new(Some(Color::white()));
        for c in "あいうえお".chars() {
            font.draw(&mut tile, c, &setting).unwrap();
            println!("{:?}", tile);
            let mut tile_new = Tile::new(Some(Color::white()));
            font.draw(&mut tile_new, c, &setting).unwrap();
            tile_rect().into_iter().for_each(|p| {
                assert_eq!(*tile.get_point(p).unwrap(), *tile_new.get_point(p).unwrap());
            });
            tile = Tile::new(Some(Color::white()));
        }
    }

    #[bench]
    fn no_cache(b: &mut Bencher) {
        let setting = FontSetting::new();
        let mut font = FontHandle::new(&MIGU[..]);
        let mut tile = Tile::new(Some(Color::white()));
        b.iter(|| {
            (0..100).for_each(|_| {
                font.draw(&mut tile, 'あ', &setting).unwrap();
                font.cache.clear();
            });
        });
    }

    #[bench]
    fn with_cache(b: &mut Bencher) {
        let setting = FontSetting::new();
        let mut font = FontHandle::new(&MIGU[..]);
        let mut tile = Tile::new(Some(Color::white()));
        b.iter(|| {
            (0..100).for_each(|_| {
                font.draw(&mut tile, 'あ', &setting).unwrap();
            });
        });
    }
}
