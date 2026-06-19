use std::path::Path;

use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, GrayImage, ImageEncoder, ImageReader, Luma, Rgb, RgbImage};
use imageproc::distance_transform::Norm;
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use imageproc::region_labelling::{connected_components, Connectivity};

const JPEG_QUALITY: u8 = 97;

pub(crate) fn open_rgb(path: &Path) -> Result<RgbImage> {
    Ok(ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?
        .to_rgb8())
}

pub(crate) fn save_rgb(img: &RgbImage, path: &Path) -> Result<()> {
    let mut buf = Vec::new();
    JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY).write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        ExtendedColorType::Rgb8,
    )?;
    std::fs::write(path, buf)?;
    Ok(())
}

// ─── Manual rotation (90° steps) ──────────────────────────────────────────────

pub fn rotate(path: &Path, clockwise: bool) -> Result<()> {
    let img = open_rgb(path)?;
    let out = if clockwise {
        image::imageops::rotate90(&img)
    } else {
        image::imageops::rotate270(&img)
    };
    save_rgb(&out, path)
}

// ─── EXIF auto-orientation (bake rotation into pixels, drop the tag) ──────────

pub fn orient(path: &Path) -> Result<()> {
    let Some(o) = crate::exif::read_orientation(path) else {
        return Ok(());
    };
    if o <= 1 {
        return Ok(());
    }
    let img = open_rgb(path)?;
    let out = match o {
        2 => image::imageops::flip_horizontal(&img),
        3 => image::imageops::rotate180(&img),
        4 => image::imageops::flip_vertical(&img),
        5 => image::imageops::rotate90(&image::imageops::flip_horizontal(&img)),
        6 => image::imageops::rotate90(&img),
        7 => image::imageops::rotate270(&image::imageops::flip_horizontal(&img)),
        8 => image::imageops::rotate270(&img),
        _ => img,
    };
    save_rgb(&out, path)
}

// ─── Auto-crop white borders ──────────────────────────────────────────────────

pub fn crop(path: &Path) -> Result<()> {
    let img = open_rgb(path)?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Ok(());
    }
    let gray = image::imageops::grayscale(&img);
    let blurred = imageproc::filter::gaussian_blur_f32(&gray, 1.5);

    let mut mask = GrayImage::new(w, h);
    for (x, y, p) in blurred.enumerate_pixels() {
        if p[0] < 240 {
            mask.put_pixel(x, y, Luma([255]));
        }
    }
    // Morphological close (~15x15) to bridge gaps in the content region.
    let mask = imageproc::morphology::close(&mask, Norm::LInf, 7);

    let (mut minx, mut miny, mut maxx, mut maxy) = (w, h, 0u32, 0u32);
    let mut any = false;
    for (x, y, p) in mask.enumerate_pixels() {
        if p[0] > 0 {
            any = true;
            minx = minx.min(x);
            miny = miny.min(y);
            maxx = maxx.max(x);
            maxy = maxy.max(y);
        }
    }
    if !any {
        return Ok(());
    }

    let m = 5i64;
    let x0 = (minx as i64 - m).max(0) as u32;
    let y0 = (miny as i64 - m).max(0) as u32;
    let x1 = ((maxx as i64 + m).min(w as i64 - 1)) as u32;
    let y1 = ((maxy as i64 + m).min(h as i64 - 1)) as u32;
    let cw = x1 - x0 + 1;
    let ch = y1 - y0 + 1;

    let area_ratio = (cw as f64 * ch as f64) / (w as f64 * h as f64);
    if !(0.10..=0.95).contains(&area_ratio) {
        return Ok(());
    }

    let cropped = image::imageops::crop_imm(&img, x0, y0, cw, ch).to_image();
    save_rgb(&cropped, path)
}

// ─── Deskew via Hough lines ───────────────────────────────────────────────────

pub fn deskew(path: &Path) -> Result<()> {
    let img = open_rgb(path)?;
    let (w, h) = img.dimensions();
    let gray = image::imageops::grayscale(&img);
    let edges = imageproc::edges::canny(&gray, 50.0, 150.0);
    let lines = imageproc::hough::detect_lines(
        &edges,
        imageproc::hough::LineDetectionOptions {
            vote_threshold: 100,
            suppression_radius: 8,
        },
    );
    if lines.is_empty() {
        return Ok(());
    }

    let mut devs: Vec<f64> = Vec::new();
    for l in &lines {
        let line_angle = l.angle_in_degrees as f64 - 90.0;
        let mut d = line_angle % 90.0;
        if d > 45.0 {
            d -= 90.0;
        } else if d < -45.0 {
            d += 90.0;
        }
        if d.abs() <= 15.0 {
            devs.push(d);
        }
    }
    if devs.is_empty() {
        return Ok(());
    }
    devs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = devs[devs.len() / 2];
    if median.abs() < 0.5 {
        return Ok(());
    }

    let theta = (median * std::f64::consts::PI / 180.0) as f32;
    let rotated = rotate_about_center(&img, -theta, Interpolation::Bilinear, Rgb([0, 0, 0]));

    let (rw, rh) = largest_rotated_rect(w as f64, h as f64, median.to_radians().abs());
    let rw = (rw.floor() as u32).min(w);
    let rh = (rh.floor() as u32).min(h);
    if rw == 0 || rh == 0 {
        return save_rgb(&rotated, path);
    }
    let x0 = (w - rw) / 2;
    let y0 = (h - rh) / 2;
    let cropped = image::imageops::crop_imm(&rotated, x0, y0, rw, rh).to_image();
    save_rgb(&cropped, path)
}

/// Largest axis-aligned rectangle that fits inside a w×h image rotated by `angle` (rad).
fn largest_rotated_rect(w: f64, h: f64, angle: f64) -> (f64, f64) {
    if w <= 0.0 || h <= 0.0 {
        return (0.0, 0.0);
    }
    let width_is_longer = w >= h;
    let (side_long, side_short) = if width_is_longer { (w, h) } else { (h, w) };
    let sin_a = angle.sin().abs();
    let cos_a = angle.cos().abs();

    if side_short <= 2.0 * sin_a * cos_a * side_long || (sin_a - cos_a).abs() < 1e-10 {
        let x = 0.5 * side_short;
        let (wr, hr) = if width_is_longer {
            (x / sin_a.max(1e-9), x / cos_a.max(1e-9))
        } else {
            (x / cos_a.max(1e-9), x / sin_a.max(1e-9))
        };
        (wr, hr)
    } else {
        let cos_2a = cos_a * cos_a - sin_a * sin_a;
        let wr = (w * cos_a - h * sin_a) / cos_2a;
        let hr = (h * cos_a - w * sin_a) / cos_2a;
        (wr, hr)
    }
}

// ─── Dust / speck removal ─────────────────────────────────────────────────────

pub fn remove_dust(path: &Path) -> Result<()> {
    let img = open_rgb(path)?;
    let (w, h) = img.dimensions();
    let gray = image::imageops::grayscale(&img);
    let mean = imageproc::filter::box_filter(&gray, 5, 5);

    let mut mask = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let g = gray.get_pixel(x, y)[0] as i32;
            let m = mean.get_pixel(x, y)[0] as i32;
            if g > m + 10 || g < m - 10 {
                mask.put_pixel(x, y, Luma([255]));
            }
        }
    }

    let labels = connected_components(&mask, Connectivity::Eight, Luma([0u8]));
    let max_label = labels.pixels().map(|p| p[0]).max().unwrap_or(0);
    if max_label == 0 {
        return Ok(());
    }
    let mut sizes = vec![0u32; (max_label + 1) as usize];
    for p in labels.pixels() {
        sizes[p[0] as usize] += 1;
    }

    let mut dust = GrayImage::new(w, h);
    let mut any = false;
    for (x, y, p) in labels.enumerate_pixels() {
        let l = p[0];
        if l != 0 && sizes[l as usize] < 50 {
            dust.put_pixel(x, y, Luma([255]));
            any = true;
        }
    }
    if !any {
        return Ok(());
    }

    let out = inpaint(&img, &dust);
    save_rgb(&out, path)
}

/// Fill masked pixels by iteratively averaging known neighbours (boundary inward).
fn inpaint(img: &RgbImage, mask: &GrayImage) -> RgbImage {
    let (w, h) = img.dimensions();
    let mut out = img.clone();
    let mut todo: Vec<(u32, u32)> = mask
        .enumerate_pixels()
        .filter(|(_, _, p)| p[0] > 0)
        .map(|(x, y, _)| (x, y))
        .collect();
    if todo.is_empty() {
        return out;
    }
    let mut filled = vec![false; (w * h) as usize];
    let idx = |x: u32, y: u32| (y * w + x) as usize;
    let is_unknown = |x: u32, y: u32, filled: &[bool]| mask.get_pixel(x, y)[0] > 0 && !filled[idx(x, y)];

    for _ in 0..40 {
        if todo.is_empty() {
            break;
        }
        let mut next = Vec::new();
        for &(x, y) in &todo {
            let mut sum = [0u32; 3];
            let mut cnt = 0u32;
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let (nx, ny) = (nx as u32, ny as u32);
                    if is_unknown(nx, ny, &filled) {
                        continue;
                    }
                    let p = out.get_pixel(nx, ny);
                    sum[0] += p[0] as u32;
                    sum[1] += p[1] as u32;
                    sum[2] += p[2] as u32;
                    cnt += 1;
                }
            }
            if cnt > 0 {
                out.put_pixel(
                    x,
                    y,
                    Rgb([
                        (sum[0] / cnt) as u8,
                        (sum[1] / cnt) as u8,
                        (sum[2] / cnt) as u8,
                    ]),
                );
                filled[idx(x, y)] = true;
            } else {
                next.push((x, y));
            }
        }
        todo = next;
    }
    out
}

// ─── Colour restoration (CLAHE on L, neutralise a/b, boost saturation) ────────

pub fn restore_color(path: &Path) -> Result<()> {
    let img = open_rgb(path)?;
    let (w, h) = img.dimensions();
    let (wu, hu) = (w as usize, h as usize);
    let n = wu * hu;

    let mut l_ch = vec![0u8; n];
    let mut a_ch = vec![0f32; n];
    let mut b_ch = vec![0f32; n];
    for (i, p) in img.pixels().enumerate() {
        let (l, a, b) = rgb_to_lab(p[0], p[1], p[2]);
        l_ch[i] = (l * 2.55).round().clamp(0.0, 255.0) as u8;
        a_ch[i] = a;
        b_ch[i] = b;
    }

    let new_l = clahe(&l_ch, wu, hu, 8, 8, 2.0);

    let a_mean: f32 = a_ch.iter().sum::<f32>() / n as f32;
    let b_mean: f32 = b_ch.iter().sum::<f32>() / n as f32;
    let shift_a = if a_mean.abs() >= 3.0 { a_mean } else { 0.0 };
    let shift_b = if b_mean.abs() >= 3.0 { b_mean } else { 0.0 };

    let mut out = RgbImage::new(w, h);
    for i in 0..n {
        let l = new_l[i] as f32 / 2.55;
        let a = a_ch[i] - shift_a;
        let b = b_ch[i] - shift_b;
        let (r, g, bl) = lab_to_rgb(l, a, b);
        let (hh, ss, vv) = rgb_to_hsv(r, g, bl);
        let (r2, g2, b2) = hsv_to_rgb(hh, (ss * 1.15).min(1.0), vv);
        out.put_pixel(
            (i % wu) as u32,
            (i / wu) as u32,
            Rgb([to_u8(r2), to_u8(g2), to_u8(b2)]),
        );
    }
    save_rgb(&out, path)
}

fn clahe(l: &[u8], w: usize, h: usize, tiles_x: usize, tiles_y: usize, clip_limit: f32) -> Vec<u8> {
    let tw = w.div_ceil(tiles_x).max(1);
    let th = h.div_ceil(tiles_y).max(1);
    let mut maps = vec![[0u8; 256]; tiles_x * tiles_y];

    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            let x0 = tx * tw;
            let y0 = ty * th;
            let x1 = ((tx + 1) * tw).min(w);
            let y1 = ((ty + 1) * th).min(h);
            let map = &mut maps[ty * tiles_x + tx];

            if x0 >= x1 || y0 >= y1 {
                for (i, m) in map.iter_mut().enumerate() {
                    *m = i as u8;
                }
                continue;
            }

            let mut hist = [0u32; 256];
            let mut count = 0u32;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    hist[l[yy * w + xx] as usize] += 1;
                    count += 1;
                }
            }

            let clip = (clip_limit * count as f32 / 256.0).max(1.0) as u32;
            let mut excess = 0u32;
            for bin in hist.iter_mut() {
                if *bin > clip {
                    excess += *bin - clip;
                    *bin = clip;
                }
            }
            let inc = excess / 256;
            let rem = (excess % 256) as usize;
            for (i, bin) in hist.iter_mut().enumerate() {
                *bin += inc;
                if i < rem {
                    *bin += 1;
                }
            }

            let mut cdf = 0u32;
            let scale = 255.0 / count as f32;
            for (i, bin) in hist.iter().enumerate() {
                cdf += *bin;
                map[i] = (cdf as f32 * scale).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    let mut out = vec![0u8; w * h];
    let clamp_x = |t: i32| t.clamp(0, tiles_x as i32 - 1) as usize;
    let clamp_y = |t: i32| t.clamp(0, tiles_y as i32 - 1) as usize;
    for y in 0..h {
        for x in 0..w {
            let v = l[y * w + x] as usize;
            let fx = x as f32 / tw as f32 - 0.5;
            let fy = y as f32 / th as f32 - 0.5;
            let tx0 = fx.floor();
            let ty0 = fy.floor();
            let dx = fx - tx0;
            let dy = fy - ty0;
            let xa = clamp_x(tx0 as i32);
            let xb = clamp_x(tx0 as i32 + 1);
            let ya = clamp_y(ty0 as i32);
            let yb = clamp_y(ty0 as i32 + 1);
            let m00 = maps[ya * tiles_x + xa][v] as f32;
            let m01 = maps[ya * tiles_x + xb][v] as f32;
            let m10 = maps[yb * tiles_x + xa][v] as f32;
            let m11 = maps[yb * tiles_x + xb][v] as f32;
            let top = m00 * (1.0 - dx) + m01 * dx;
            let bot = m10 * (1.0 - dx) + m11 * dx;
            out[y * w + x] = (top * (1.0 - dy) + bot * dy).round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

// ─── Colour-space helpers ─────────────────────────────────────────────────────

fn to_u8(v: f32) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn rgb_to_lab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rl = srgb_to_linear(r as f32 / 255.0);
    let gl = srgb_to_linear(g as f32 / 255.0);
    let bl = srgb_to_linear(b as f32 / 255.0);
    // sRGB -> XYZ (D65)
    let x = (rl * 0.4124 + gl * 0.3576 + bl * 0.1805) / 0.95047;
    let y = rl * 0.2126 + gl * 0.7152 + bl * 0.0722;
    let z = (rl * 0.0193 + gl * 0.1192 + bl * 0.9505) / 1.08883;
    let f = |t: f32| {
        if t > 0.008856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

fn lab_to_rgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let fy = (l + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;
    let inv = |t: f32| {
        let t3 = t * t * t;
        if t3 > 0.008856 {
            t3
        } else {
            (t - 16.0 / 116.0) / 7.787
        }
    };
    let x = inv(fx) * 0.95047;
    let y = inv(fy);
    let z = inv(fz) * 1.08883;
    let rl = x * 3.2406 + y * -1.5372 + z * -0.4986;
    let gl = x * -0.9689 + y * 1.8758 + z * 0.0415;
    let bl = x * 0.0557 + y * -0.2040 + z * 1.0570;
    (
        linear_to_srgb(rl).clamp(0.0, 1.0),
        linear_to_srgb(gl).clamp(0.0, 1.0),
        linear_to_srgb(bl).clamp(0.0, 1.0),
    )
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { d / max };
    (h, s, max)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_jpeg(path: &Path) {
        let mut img = RgbImage::new(200, 150);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = if x < 20 || y < 20 || x >= 180 || y >= 130 {
                Rgb([255, 255, 255]) // white border
            } else {
                Rgb([(x % 256) as u8, (y % 256) as u8, 128]) // gradient content
            };
        }
        img.put_pixel(100, 75, Rgb([0, 0, 0])); // a dust speck
        save_rgb(&img, path).unwrap();
    }

    fn assert_valid(path: &Path) {
        let img = open_rgb(path).unwrap();
        assert!(img.width() > 0 && img.height() > 0);
    }

    #[test]
    fn classical_steps_run_on_real_pixels() {
        let dir = std::env::temp_dir().join("pa_imaging_test");
        std::fs::create_dir_all(&dir).unwrap();
        for step in [
            "crop",
            "deskew",
            "restore_color",
            "remove_dust",
            "rotate",
            "orient",
        ] {
            let p = dir.join(format!("{step}.jpg"));
            make_test_jpeg(&p);
            match step {
                "crop" => crop(&p).unwrap(),
                "deskew" => deskew(&p).unwrap(),
                "restore_color" => restore_color(&p).unwrap(),
                "remove_dust" => remove_dust(&p).unwrap(),
                "rotate" => rotate(&p, true).unwrap(),
                "orient" => orient(&p).unwrap(),
                _ => {}
            }
            assert_valid(&p);
        }
    }

    #[test]
    fn crop_removes_white_border() {
        let dir = std::env::temp_dir().join("pa_imaging_test");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("crop_border.jpg");
        make_test_jpeg(&p);
        crop(&p).unwrap();
        let out = open_rgb(&p).unwrap();
        // Content was 160x110 inside a 200x150 frame; crop should shrink it.
        assert!(out.width() < 200 && out.height() < 150);
    }
}
