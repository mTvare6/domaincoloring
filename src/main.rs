#![allow(unused)]
use num::{Complex};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const W: usize = 2048;
const H: usize = 2048;
const SUPERSAMPLING: usize = 1;
const SW: usize = SUPERSAMPLING * W;
const SH: usize = SUPERSAMPLING * H;

mod colormaps;
use crate::colormaps::*;


fn f(z: Complex<f64>) -> Complex<f64> {
    return ((z-1.)/(z)).sin().powu(3);
}


fn main() {

    let maps = get_all_color_maps();
    let path = Path::new(r"image.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, W as u32, H as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
    let source_chromaticities = png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000),
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header().unwrap();

    let mut data: [u8; SW * SH * 4] = [0; SW * SH * 4];
    let (x0, y0) = pixel_coordinates(0, 0);
    let (x1, y1) = pixel_coordinates(SW - 1, SH - 1);
    let dx = (x1 - x0) / (SW - 1) as f64;
    let dy = (y1 - y0) / (SH - 1) as f64;

    let mut im = y0;
    for py in 0..SH {
        let mut re = x0;
        for px in 0..SW {
            let z = f(Complex { re, im });
            let c = complex_color(z, &maps["inferno"]);
            let i = 4 * ( py * W + px);
            data[i..i + 4].copy_from_slice(&c);
            re += dx;
        }
        im += dy
    }
    writer.write_image_data(&data).unwrap();
}
