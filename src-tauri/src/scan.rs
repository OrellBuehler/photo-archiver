use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use sqlx::SqlitePool;
use walkdir::WalkDir;

struct ScannedFile {
    source_path: String,
    filename: String,
    scan_id: Option<String>,
    file_size: Option<i64>,
    width: Option<i64>,
    height: Option<i64>,
    year: Option<i64>,
    month: Option<i64>,
    title: Option<String>,
}

/// Image extensions we ingest. The `image` crate decodes all of these by
/// default. HEIC/HEIF need a system libheif and are intentionally excluded.
pub const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "tif", "tiff", "webp", "bmp", "gif",
];

fn is_image_ext(ext: &str) -> bool {
    IMAGE_EXTS.contains(&ext)
}

fn scan_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"SCAN_\d+").unwrap())
}

fn title_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Extension-agnostic: any image extension, not just jpg.
    RE.get_or_init(|| Regex::new(r"(?i)SCAN_\d+_(.+)\.[A-Za-z0-9]+$").unwrap())
}

/// Parse a folder/segment into a month number, accepting a numeric value
/// (`3`, `03`), German names, or English names (full or 3-letter).
fn parse_month(s: &str) -> Option<i64> {
    let t = s.trim().to_lowercase();
    if let Ok(n) = t.parse::<i64>() {
        return (1..=12).contains(&n).then_some(n);
    }
    let m = match t.as_str() {
        "januar" | "january" | "jan" => 1,
        "februar" | "february" | "feb" => 2,
        "märz" | "maerz" | "march" | "mar" | "mär" => 3,
        "april" | "apr" => 4,
        "mai" | "may" => 5,
        "juni" | "june" | "jun" => 6,
        "juli" | "july" | "jul" => 7,
        "august" | "aug" => 8,
        "september" | "sep" | "sept" => 9,
        "oktober" | "october" | "okt" | "oct" => 10,
        "november" | "nov" => 11,
        "dezember" | "december" | "dez" | "dec" => 12,
        _ => return None,
    };
    Some(m)
}

fn collect_files(source_dir: &Path) -> Vec<ScannedFile> {
    let mut out = Vec::new();
    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if !ext.as_deref().map(is_image_ext).unwrap_or(false) {
            continue;
        }
        let Ok(rel) = path.strip_prefix(source_dir) else {
            continue;
        };
        let source_path = rel.to_string_lossy().replace('\\', "/");
        let filename = path.file_name().unwrap_or_default().to_string_lossy().into_owned();

        let comps: Vec<String> = rel
            .components()
            .filter_map(|c| match c {
                Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect();
        let mut year = None;
        let mut month = None;
        if comps.len() >= 2 {
            if let Ok(y) = comps[0].parse::<i64>() {
                year = Some(y);
                if comps.len() >= 3 {
                    month = parse_month(&comps[1]);
                }
            }
        }

        let scan_id = scan_id_re().find(&filename).map(|m| m.as_str().to_string());
        let title = title_re()
            .captures(&filename)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string());

        let file_size = entry.metadata().ok().map(|m| m.len() as i64);
        let (width, height) = match image::image_dimensions(path) {
            Ok((w, h)) => (Some(w as i64), Some(h as i64)),
            Err(_) => (None, None),
        };

        out.push(ScannedFile {
            source_path,
            filename,
            scan_id,
            file_size,
            width,
            height,
            year,
            month,
            title,
        });
    }
    out
}

/// Walk the source directory, extract metadata, and upsert all JPEGs.
/// Returns the number of files seen.
pub async fn scan_source(pool: &SqlitePool, source_dir: PathBuf) -> Result<usize> {
    let files = tauri::async_runtime::spawn_blocking(move || collect_files(&source_dir)).await?;

    let mut tx = pool.begin().await?;
    for f in &files {
        sqlx::query(
            r#"
            INSERT INTO images
              (source_path, filename, scan_id, file_size, width, height, year, month, title)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(source_path) DO UPDATE SET
              filename = excluded.filename,
              file_size = excluded.file_size,
              width = excluded.width,
              height = excluded.height,
              updated_at = datetime('now')
            "#,
        )
        .bind(&f.source_path)
        .bind(&f.filename)
        .bind(&f.scan_id)
        .bind(f.file_size)
        .bind(f.width)
        .bind(f.height)
        .bind(f.year)
        .bind(f.month)
        .bind(&f.title)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(files.len())
}
