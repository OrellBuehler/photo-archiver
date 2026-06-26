use std::path::Path;

use anyhow::Result;

use crate::exif;
use crate::imaging::{open_rgb, save_rgb};

/// Copy a source image into the organized output tree and tag it.
/// Returns the organized path relative to `output_dir`.
///
/// `album` is an optional user-assigned top-level folder (e.g. "Wedding").
/// Non-JPEG sources are decoded and re-encoded to JPEG so the `.jpg` output is
/// always a valid JPEG rather than foreign bytes under a misleading extension.
pub fn organize(
    source_abs: &Path,
    source_rel: &str,
    output_dir: &Path,
    album: Option<&str>,
    year: Option<i64>,
    month: Option<i64>,
    base_name: &str,
) -> Result<String> {
    let album = album.map(str::trim).filter(|a| !a.is_empty());
    let top = match album {
        Some(a) => format!("organized/{}", sanitize_folder(a)),
        None => "organized".to_string(),
    };
    let rel = match (album, year, month) {
        (_, Some(y), Some(m)) => format!("{top}/{y}/{y}-{m:02}_{base_name}.jpg"),
        (_, Some(y), None) => format!("{top}/{y}/{y}_{base_name}.jpg"),
        (Some(_), None, _) => format!("{top}/{base_name}.jpg"),
        (None, None, _) => format!("organized/unsorted/{base_name}.jpg"),
    };
    let dest = output_dir.join(&rel);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let is_jpeg = source_abs
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
        .unwrap_or(false);
    if is_jpeg {
        std::fs::copy(source_abs, &dest)?;
    } else {
        // Decode the foreign format and write an honest JPEG.
        save_rgb(&open_rgb(source_abs)?, &dest)?;
    }

    exif::write_provenance(&dest, source_rel, year, month);
    Ok(rel)
}

/// Keep an album folder name to a single safe path segment.
fn sanitize_folder(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    let cleaned = cleaned.trim_matches(|c: char| c == ' ' || c == '.').to_string();
    if cleaned.is_empty() {
        "album".to_string()
    } else {
        cleaned
    }
}
