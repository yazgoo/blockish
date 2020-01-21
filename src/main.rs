extern crate image;

use std::env;
use image::GenericImageView;
use image::imageops::FilterType;
use blockish::render;

fn main() {

    let args: Vec<String> = env::args().collect();
    let img = image::open(&args[1]).unwrap();
    let width = args[2].parse::<u32>().unwrap() * 8;
    let height = img.height() * width / img.width();
    let subimg  = img.resize(width, height, FilterType::Nearest);
    let raw: Vec<u8> = subimg.to_rgb().into_raw();
    let raw_slice = raw.as_slice();
    let width = subimg.width();
    let height = subimg.height();

    render(width, height, &|x, y| {
        let start = ((y * width + x)*3) as usize;
        (raw_slice[start], raw_slice[start + 1], raw_slice[start + 2])
    });
}


