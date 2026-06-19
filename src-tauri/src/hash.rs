use std::path::Path;

use anyhow::Result;
use image::ImageReader;
use image_hasher::{HasherConfig, ImageHash};

/// Perceptual hash of an image, base64-encoded for storage.
pub fn phash(path: &Path) -> Result<String> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    let hasher = HasherConfig::new().to_hasher();
    Ok(hasher.hash_image(&img).to_base64())
}

/// Hamming distance between two stored hashes, if both decode.
pub fn distance(a: &str, b: &str) -> Option<u32> {
    let ha = ImageHash::<Box<[u8]>>::from_base64(a).ok()?;
    let hb = ImageHash::<Box<[u8]>>::from_base64(b).ok()?;
    Some(ha.dist(&hb))
}
