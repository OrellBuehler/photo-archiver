use std::path::Path;

use anyhow::Result;

use crate::exif;

/// Copy a source image into the organized output tree and tag it.
/// Returns the organized path relative to `output_dir`.
pub fn organize(
    source_abs: &Path,
    source_rel: &str,
    output_dir: &Path,
    year: Option<i64>,
    month: Option<i64>,
    base_name: &str,
) -> Result<String> {
    let rel = match (year, month) {
        (Some(y), Some(m)) => format!("organized/{y}/{y}-{m:02}_{base_name}.jpg"),
        (Some(y), None) => format!("organized/{y}/{y}_{base_name}.jpg"),
        _ => format!("organized/unsorted/{base_name}.jpg"),
    };
    let dest = output_dir.join(&rel);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source_abs, &dest)?;
    exif::write_provenance(&dest, source_rel, year, month);
    Ok(rel)
}
