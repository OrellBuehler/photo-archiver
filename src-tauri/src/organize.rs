use std::path::Path;

use anyhow::Result;

use crate::exif;
use crate::imaging::{open_rgb, save_rgb};

/// Where a photo belongs in the output tree, relative to `output_dir`.
///
/// `album` is an optional user-assigned top-level folder (e.g. "Wedding").
pub fn rel_path(album: Option<&str>, year: Option<i64>, month: Option<i64>, base_name: &str) -> String {
    let album = album.map(str::trim).filter(|a| !a.is_empty());
    let top = match album {
        Some(a) => format!("organized/{}", sanitize_folder(a)),
        None => "organized".to_string(),
    };
    match (album, year, month) {
        (_, Some(y), Some(m)) => format!("{top}/{y}/{y}-{m:02}_{base_name}.jpg"),
        (_, Some(y), None) => format!("{top}/{y}/{y}_{base_name}.jpg"),
        (Some(_), None, _) => format!("{top}/{base_name}.jpg"),
        (None, None, _) => format!("organized/unsorted/{base_name}.jpg"),
    }
}

/// Copy a source image into the organized output tree and tag it.
/// Returns the organized path relative to `output_dir`.
///
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
    let rel = rel_path(album, year, month, base_name);
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

/// Move a file within the output tree, creating the destination folder and
/// pruning the folders it leaves empty.
pub fn move_file(output_dir: &Path, from_rel: &str, to_rel: &str) -> Result<()> {
    let from = output_dir.join(from_rel);
    let to = output_dir.join(to_rel);
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if std::fs::rename(&from, &to).is_err() {
        std::fs::copy(&from, &to)?;
        std::fs::remove_file(&from)?;
    }
    // `remove_dir` refuses non-empty folders, so this only prunes empties.
    let mut dir = from.parent();
    while let Some(d) = dir {
        if d == output_dir || std::fs::remove_dir(d).is_err() {
            break;
        }
        dir = d.parent();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rel_path_layout() {
        assert_eq!(rel_path(None, Some(1978), Some(6), "a_1"), "organized/1978/1978-06_a_1.jpg");
        assert_eq!(rel_path(None, Some(1978), None, "a_1"), "organized/1978/1978_a_1.jpg");
        assert_eq!(rel_path(None, None, Some(6), "a_1"), "organized/unsorted/a_1.jpg");
        assert_eq!(rel_path(Some(" Wedding/78 "), Some(1978), None, "a_1"), "organized/Wedding_78/1978/1978_a_1.jpg");
        assert_eq!(rel_path(Some("Wedding"), None, None, "a_1"), "organized/Wedding/a_1.jpg");
    }

    #[test]
    fn move_file_prunes_empty_folders() {
        let out = std::env::temp_dir().join(format!("pa_move_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        std::fs::create_dir_all(out.join("organized/unsorted")).unwrap();
        std::fs::write(out.join("organized/unsorted/a.jpg"), b"x").unwrap();
        move_file(&out, "organized/unsorted/a.jpg", "organized/1978/1978_a.jpg").unwrap();
        assert!(out.join("organized/1978/1978_a.jpg").exists());
        assert!(!out.join("organized/unsorted").exists());
        assert!(out.join("organized").exists());
        std::fs::remove_dir_all(&out).unwrap();
    }
}
