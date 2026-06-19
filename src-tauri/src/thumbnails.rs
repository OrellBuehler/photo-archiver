use std::path::Path;

use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageReader};

fn generate(src: &Path, dest: &Path, size: u32) -> Result<()> {
    let img = ImageReader::open(src)?.with_guessed_format()?.decode()?;
    // Preserves aspect ratio, fitting within `size` x `size`.
    let thumb = img.thumbnail(size, size).to_rgb8();
    let (w, h) = thumb.dimensions();

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, 85);
    encoder.write_image(thumb.as_raw(), w, h, ExtendedColorType::Rgb8)?;
    std::fs::write(dest, &buf)?;
    Ok(())
}

/// Generate the thumbnail if it is missing, then return its JPEG bytes.
pub fn ensure_thumbnail(src: &Path, dest: &Path, size: u32) -> Result<Vec<u8>> {
    if !dest.exists() {
        generate(src, dest, size)?;
    }
    Ok(std::fs::read(dest)?)
}
