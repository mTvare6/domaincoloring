use num::Complex;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use rayon::prelude::*;

const SUPERSAMPLING: usize = 1;
const FOVY: f64 = 1.0;

const W: usize = 4096;
const H: usize = 4096;
const SW: usize = SUPERSAMPLING * W;
const SH: usize = SUPERSAMPLING * H;
const CENTER_REAL: f64 = 0.;
const CENTER_IMAG: f64 = 0.;
const ASPECT_RATIO: f64 = W as f64 / H as f64;
const HALF_FOVY: f64 = FOVY / 2.0;

mod colormaps;
use crate::colormaps::*;

fn f(z: Complex<f64>) -> Complex<f64> {
    return z.finv().sin().finv();
}

pub fn pixel_coordinates(px: usize, py: usize) -> (f64, f64) {
    // first part shifts to 2x-1, making 0->-1,1->1, 0.5->0
    // going from graphical coordinates to cartesian
    // then it's multiplied and origin is shifted
    let x = ((px as f64 / (SW - 1) as f64) * 2. - 1.) * ASPECT_RATIO * HALF_FOVY + CENTER_REAL;
    let y = (((SH - py - 1) as f64 / (SH - 1) as f64) * 2. - 1.) * HALF_FOVY + CENTER_IMAG;
    return (x, y);
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


    data.par_chunks_mut(4).enumerate().for_each(|(i, chunk)| {
        let py = i/SW;
        let px = i - py*SW;
        let z = f(Complex { re: px as f64 * dx + x0, im: py as f64 * dy + y0 });
        let c = complex_color(z, &maps["inferno"]);
        chunk.copy_from_slice(&c);

    });
    writer.write_image_data(&data).unwrap();
}
