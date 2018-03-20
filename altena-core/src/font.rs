use std::collections::HashMap;

use std::cmp;
use std::fmt;
use std::error::Error;
use rusttype::{point, Font, FontCollection, Point, Scale};

use rect_iter::{GetMut2D, IndexError, ToPoint};
use tile::tiletypes::*;
use tile::{Alpha, Blend, Color};
use tuple_map::TupleMap2;

#[derive(Copy, Clone, Debug)]
enum FontError {
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

struct FontHandle<'a> {
    font: Font<'a>,
}

impl<'a> FontHandle<'a> {
    fn new(font: &'a [u8]) -> Self {
        let collection = FontCollection::from_bytes(font);
        let font = collection.into_font().expect("Invalid font data");
        Self { font: font }
    }

    fn draw<B, I>(&mut self, buf: &mut B, c: char, setting: &FontSetting) -> Result<(), FontError>
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
        if let Some(bbox) = glyph.pixel_bounding_box() {
            println!("{:?}", bbox);
            let offset = (bbox.min.x, bbox.min.y).map(|i| i as u32);
            glyph.draw(|x, y, v| {
                let mut alpha = Alpha::from_f32(v);
                if alpha.0 < 5 {
                    return;
                }
                alpha.plus(3);
                match buf.get_mut_xy_r(x, y) {
                    Ok(b) => b.blend(setting.color, alpha),
                    Err(id) => res = Err(FontError::Index(id)),
                }
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct FontSetting {
    color: Color,
    scale: Scale,
    scale_usize: usize,
    start: Point<f32>,
}

impl FontSetting {
    fn color(&mut self, c: Color) -> &mut Self {
        self.color = c;
        self
    }
    fn new() -> Self {
        FontSetting {
            color: Color::black(),
            scale: Scale::uniform(TILE_SIZE as f32),
            scale_usize: TILE_SIZE as usize,
            start: point(0.0, 0.0),
        }
    }
    fn scale<T: Into<i32>>(&mut self, s: T) -> &mut Self {
        let s = s.into();
        self.scale = Scale::uniform(s as f32);
        self.scale_usize = s as usize;
        self
    }
    fn start<P, T>(&mut self, p: P) -> &mut Self
    where
        P: ToPoint<T>,
        T: Into<f32>,
    {
        let (x, y) = p.to_point().map(T::into);
        self.start = point(x, y);
        self
    }
}

struct FontCache {
    inner: Vec<u8>,
    size: usize,
}

mod font_test {
    use super::*;
    use tile::Tile;
    #[test]
    fn draw_tile() {
        let setting = FontSetting::new();
        let migu = include_bytes!("../../assets/migu-1m-regular.ttf");
        let mut font = FontHandle::new(&migu[..]);
        let mut tile = Tile::new(Some(Color::white()));
        font.draw(&mut tile, 'あ', &setting).unwrap();
        println!("{:?}", tile);
    }
}
