import os
import shutil
from app.config import settings
from app.utils.exif import write_exif


def organize_image(source_path: str, scan_id: str | None, year: int | None, month: int | None) -> str:
    """Copy source to organized path, set EXIF, return relative organized path."""
    full_source = os.path.join(settings.source_dir, source_path)

    name = scan_id or os.path.splitext(os.path.basename(source_path))[0]

    if year and month:
        rel_path = f"organized/{year}/{year}-{month:02d}_{name}.jpg"
    elif year:
        rel_path = f"organized/{year}/{year}_{name}.jpg"
    else:
        rel_path = f"organized/unsorted/{name}.jpg"

    full_dest = os.path.join(settings.output_dir, rel_path)
    os.makedirs(os.path.dirname(full_dest), exist_ok=True)
    shutil.copy2(full_source, full_dest)

    tags = {"UserComment": f"Source: {source_path}"}
    if year:
        month_val = month or 1
        tags["DateTimeOriginal"] = f"{year}:{month_val:02d}:01 00:00:00"

    try:
        write_exif(full_dest, tags)
    except Exception:
        pass

    return rel_path
