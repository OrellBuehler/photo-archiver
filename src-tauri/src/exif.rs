use std::path::Path;

/// Read the EXIF orientation tag (1..8) if present.
pub fn read_orientation(path: &Path) -> Option<u16> {
    let file = std::fs::File::open(path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    let exif = ::exif::Reader::new()
        .read_from_container(&mut reader)
        .ok()?;
    let field = exif.get_field(::exif::Tag::Orientation, ::exif::In::PRIMARY)?;
    field.value.get_uint(0).map(|v| v as u16)
}

/// Best-effort: write provenance (source path) and a synthetic capture date
/// while preserving any EXIF already present on the file.
pub fn write_provenance(path: &Path, source: &str, year: Option<i64>, month: Option<i64>) {
    use little_exif::exif_tag::ExifTag;
    use little_exif::metadata::Metadata;

    // Start from the file's existing metadata so we don't discard tags the
    // scanner or a previous run wrote; fall back to empty if it has none.
    let mut md = Metadata::new_from_path(path).unwrap_or_else(|_| Metadata::new());
    md.set_tag(ExifTag::ImageDescription(format!("Source: {source}")));
    md.set_tag(ExifTag::Software("Photo Archiver".to_string()));
    if let Some(y) = year {
        let m = month.unwrap_or(1);
        md.set_tag(ExifTag::DateTimeOriginal(format!(
            "{y:04}:{m:02}:01 00:00:00"
        )));
    }
    let _ = md.write_to_file(path);
}
