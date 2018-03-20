use image::{open, DynamicImage, RgbaImage};
use std::path::Path;
use frame::{Frame, MeshTree};
use font::{FontHandle, FontSetting};

pub(crate) fn load_img(file_name: &str) -> RgbaImage {
    let p = Path::new(file_name);
    let res = open(p).unwrap();
    match res {
        DynamicImage::ImageRgba8(r) => r,
        x => x.to_rgba(),
    }
}

pub(crate) fn load_frame(file_name: &str) -> Frame {
    let img = load_img(file_name);
    Frame::from_buf(&img, file_name).unwrap()
}

pub(crate) fn load_mesh(file_name: &str) -> MeshTree {
    let img = load_img(file_name);
    MeshTree::from_buf(&img).unwrap()
}
