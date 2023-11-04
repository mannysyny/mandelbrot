use std::env;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use num_complex::Complex;
use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

enum FractalType {
    Mandelbrot,
    Julia(Complex<f64>)
}

fn compute_color(z: Complex<f64>, c: Complex<f64>, max_iter: u32, color_scheme: ColorScheme) -> Rgb<u8> {
    let mut i = 0;
    let mut w = z;
    while i < max_iter && w.norm() <= 2.0 {
        w = w * w + c;
        i += 1;
    }
    let color = match color_scheme {
        ColorScheme::BlackAndWhite => {
            if i == max_iter {
                Rgb([0, 0, 0])
            } else {
                let intensity = (i as f64 / max_iter as f64) * 255.0;
                Rgb([intensity as u8, intensity as u8, intensity as u8])
            }
        },
        ColorScheme::Rainbow => {
            if i == max_iter {
                Rgb([0, 0, 0])
            } else {
                let r = (i as f64 / max_iter as f64).powf(0.3);
                let g = (i as f64 / max_iter as f64).powf(0.5);
                let b = 1.0 - (i as f64 / max_iter as f64).powf(0.7);
                Rgb([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8])
            }
        },
        ColorScheme::Grayscale => {
            if i == max_iter {
                Rgb([0, 0, 0])
            } else {
                let intensity = (i as f64 / max_iter as f64) * 255.0;
                Rgb([intensity as u8, intensity as u8, intensity as u8])
            }
        }
    };
    color
}

enum ColorScheme {
    BlackAndWhite,
    Rainbow,
    Grayscale
}

fn draw_fractal(width: u32, height: u32, max_iter: u32, scale: f64, fractal_type: FractalType, zoom_level: f64, pan_position: (f64, f64)) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut imgbuf = ImageBuffer::new(width, height);
    let (w, h) = (width as f64, height as f64);
    let (capture_w, capture_h) = ((width as f64 / zoom_level) as u32, (height as f64 / zoom_level) as u32);
    let (pan_x, pan_y) = pan_position;
    let (view_w, view_h) = (capture_w as f64 / w * scale, capture_h as f64 / h * scale);
    let (view_x, view_y) = (pan_x - view_w / 2.0, pan_y - view_h / 2.0);
    let pb = ProgressBar::new((capture_w * capture_h) as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)")
        .progress_chars("#>-"));
    imgbuf.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
        let cx = (x as f64 - 0.5 * capture_w as f64) * scale / w + view_x;
        let cy = (y as f64 - 0.5 * capture_h as f64) * scale / h + view_y;
        let c = Complex::new(cx, cy);
        let color = match fractal_type {
            FractalType::Mandelbrot => compute_color(Complex::new(0.0, 0.0), c, max_iter),
            FractalType::Julia(z) => compute_color(z, c, max_iter)
        };
        *pixel = color;
        pb.inc(1);
    });
    pb.finish_with_message("done");
    imgbuf
}

fn parse_resolution(resolution: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = resolution.split('x').collect();
    if parts.len() != 2 {
        return None;
    }
    let width = u32::from_str(parts[0]).ok()?;
    let height = u32::from_str(parts[1]).ok()?;
    Some((width, height))
}

fn parse_complex_number(s: &str) -> Option<Complex<f64>> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let re = f64::from_str(parts[0]).ok()?;
    let im = f64::from_str(parts[1]).ok()?;
    Some(Complex::new(re, im))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 8 {
        println!("Usage: {} <output_file> <width>x<height> <capture_width>x<capture_height> <max_iter> <scale> <fractal_type> <c>", args[0]);
        return Ok(());
    }
    let output_file = &args[1];
    let (width, height) = parse_resolution(&args[2]).ok_or("Invalid resolution")?;
    let (capture_width, capture_height) = parse_resolution(&args[3]).ok_or("Invalid capture size")?;
    let max_iter = u32::from_str(&args[4]).map_err(|_| "Invalid max_iter")?;
    let scale = f64::from_str(&args[5]).map_err(|_| "Invalid scale")?;
    let fractal_type = match &args[6][..] {
        "mandelbrot" => FractalType::Mandelbrot,
        "julia" => {
            let c = parse_complex_number(&args[7]).ok_or("Invalid complex number")?;
            FractalType::Julia(c)
        },
        _ => return Err("Invalid fractal type".into())
    };
    let imgbuf = draw_fractal(capture_width, capture_height, max_iter, scale, fractal_type);
    let resized = image::imageops::resize(&imgbuf, width, height, image::imageops::FilterType::Lanczos3);
    let mut file = File::create(output_file)?;
    resized.save_with_format(&mut file, image::ImageFormat::PNG)?;
    Ok(())
}

