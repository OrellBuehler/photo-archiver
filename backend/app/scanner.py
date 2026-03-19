import os
import re
from app.config import settings
from app.db import get_db

GERMAN_MONTHS = {
    "Januar": 1, "Februar": 2, "März": 3, "April": 4,
    "Mai": 5, "Juni": 6, "Juli": 7, "August": 8,
    "September": 9, "Oktober": 10, "November": 11, "Dezember": 12,
}

SCAN_RE = re.compile(r"SCAN_(\d+)")
TITLE_RE = re.compile(r"SCAN_\d+_(.+)\.jpg$", re.IGNORECASE)


async def scan_source():
    source = settings.source_dir
    images = []

    for root, dirs, files in os.walk(source):
        for f in files:
            if not f.lower().endswith(".jpg"):
                continue

            full_path = os.path.join(root, f)
            rel_path = os.path.relpath(full_path, source)
            parts = rel_path.replace("\\", "/").split("/")

            year = None
            month = None

            if len(parts) >= 2:
                try:
                    year = int(parts[0])
                except ValueError:
                    pass

            if len(parts) >= 3 and year is not None:
                month_part = parts[1]
                if "_" in month_part:
                    month_name = month_part.split("_", 1)[1]
                    month = GERMAN_MONTHS.get(month_name)

            scan_match = SCAN_RE.search(f)
            scan_id = scan_match.group(1) if scan_match else None

            title_match = TITLE_RE.search(f)
            title = title_match.group(1) if title_match else None

            file_size = os.path.getsize(full_path)

            width, height = None, None
            try:
                from PIL import Image
                with Image.open(full_path) as img:
                    width, height = img.size
            except Exception:
                pass

            images.append((rel_path, f, scan_id, file_size, width, height, year, month, title))

    async with get_db() as db:
        for img in images:
            await db.execute("""
                INSERT INTO images (source_path, filename, scan_id, file_size, width, height, year, month, title)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(source_path) DO UPDATE SET
                    file_size=excluded.file_size,
                    width=excluded.width,
                    height=excluded.height,
                    updated_at=datetime('now')
            """, img)
        await db.commit()

    return len(images)
