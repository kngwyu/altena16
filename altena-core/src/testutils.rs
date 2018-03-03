use image::{open, DynamicImage, RgbaImage};
use std::path::Path;

#[cfg(test)]
pub(crate) fn load_img(file_name: &str) -> RgbaImage {
    let p = Path::new(file_name);
    let res = open(p).unwrap();
    match res {
        DynamicImage::ImageRgba8(r) => r,
        x => x.to_rgba(),
    }
}
