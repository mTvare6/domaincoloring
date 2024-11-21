use num::Complex;
use rayon::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

mod colormaps;
use crate::colormaps::*;

fn f(z: Complex<f64>) -> Complex<f64> {
    return z.finv().sin().finv();
}

fn pixel_coordinates(px: usize, py: usize, config: &Config) -> (f64, f64) {
    // first part shifts to 2x-1, making 0->-1,1->1, 0.5->0
    // going from graphical coordinates to cartesian
    // then it's multiplied and origin is shifted
    let x = ((px as f64 / (config.sw - 1) as f64) * 2. - 1.) * config.aspect_ratio * config.half_fovy + config.center_real;
    let y = (((config.sh - py - 1) as f64 / (config.sh - 1) as f64) * 2. - 1.) * config.half_fovy + config.center_imag;
    return (x, y);
}

struct Config {
    map: ColorMap,
    w: usize,
    h: usize,
    #[allow(unused)]
    supersampling: usize,
    center_real: f64,
    center_imag: f64,
    aspect_ratio: f64,
    outfile: String,
    half_fovy: f64,
    sw: usize,
    sh: usize,
}

fn parse_args() -> Result<Config, lexopt::Error> {
    use lexopt::prelude::*;

    let maps = get_all_color_maps();

    let mut w = 4096;
    let mut h = 4096;
    let mut center_real = 0.;
    let mut center_imag = 0.;
    let mut scheme = None;

    let mut outfile = None;
    let mut fovy = 1.0;
    let mut supersampling = 1;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('f') | Long("fovy") => {
                fovy = parser.value()?.parse()?;
            }
            Short('r') | Long("center-real") => {
                center_real = parser.value()?.parse()?;
            }
            Short('i') | Long("center-imag") => {
                center_imag = parser.value()?.parse()?;
            }
            Short('W') | Long("width") => {
                w = parser.value()?.parse()?;
            }
            Short('H') | Long("height") => {
                h = parser.value()?.parse()?;
            }
            Short('s') | Long("supersample") => {
                supersampling = parser.value()?.parse()?;
            }
            Short('c') | Long("color") if scheme.is_none() => {
                scheme = Some(parser.value()?.string()?);
            }
            //Short('x') | Long("hex") => {
            //    h = parser.value()?.parse()?;
            //}
            Value(val) if outfile.is_none() => {
                outfile = Some(val.string()?);
            }
            Short('h') | Long("help") => {
                println!("Usage: tinydc [-f|--fovy=NUM] [-r|--center-real=NUM] [-i|--center-imag=NUM] [-W|--width=NUM] [-H|--height=NUM] [-s|--supersample=NUM] [-c|--color=SCHEME] out.png");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }


    let map = if let Some(name) = scheme {
        maps.get(&name).unwrap_or(&maps["inferno"]).clone()
    } else {
        maps["inferno"].clone()
    };

    Ok(Config {
        map,
        w,
        h,
        supersampling,
        center_real,
        center_imag,
        aspect_ratio:  w as f64/ h as f64,
        outfile: outfile.ok_or("missing argument output file")?,
        half_fovy: fovy/2.,
        sw: supersampling*w,
        sh: supersampling*h
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let config = parse_args()?;
    let path = Path::new(&config.outfile);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, config.w as u32, config.h as u32);
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

    //let mut data: [u8; config.sw * config.sh * 4] = [0; config.sw * config.sh * 4];
    let mut data = vec![0; config.sw*config.sh*4];
    let (x0, y0) = pixel_coordinates(0, 0, &config);
    let (x1, y1) = pixel_coordinates(config.sw - 1, config.sh - 1, &config);
    let dx = (x1 - x0) / (config.sw - 1) as f64;
    let dy = (y1 - y0) / (config.sh - 1) as f64;

    data.par_chunks_mut(4).enumerate().for_each(|(i, chunk)| {
        let py = i / config.sw;
        let px = i - py * config.sw;
        let z = f(Complex {
            re: px as f64 * dx + x0,
            im: py as f64 * dy + y0,
        });
        let c = complex_color(z, &config.map);
        chunk.copy_from_slice(&c);
    });
    writer.write_image_data(&data).unwrap();
    Ok(())
}
