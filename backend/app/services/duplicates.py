import asyncio
import imagehash
from PIL import Image as PILImage
import os
from app.config import settings
from app.db import get_db


async def compute_phashes():
    async with get_db() as db:
        cursor = await db.execute("SELECT id, source_path FROM images WHERE phash IS NULL")
        rows = [dict(r) for r in await cursor.fetchall()]

    for row in rows:
        path = os.path.join(settings.source_dir, row["source_path"])
        try:
            h = await asyncio.to_thread(_compute_hash, path)
            async with get_db() as db:
                await db.execute("UPDATE images SET phash = ? WHERE id = ?", (h, row["id"]))
                await db.commit()
        except Exception:
            pass


def _compute_hash(path: str) -> str:
    with PILImage.open(path) as img:
        return str(imagehash.phash(img))


async def find_duplicates(threshold: int = 6):
    async with get_db() as db:
        cursor = await db.execute("SELECT id, phash FROM images WHERE phash IS NOT NULL")
        rows = [dict(r) for r in await cursor.fetchall()]

    groups = []
    used = set()

    for i, a in enumerate(rows):
        if a["id"] in used:
            continue
        ha = imagehash.hex_to_hash(a["phash"])
        group = [a["id"]]
        for b in rows[i + 1:]:
            if b["id"] in used:
                continue
            hb = imagehash.hex_to_hash(b["phash"])
            if ha - hb <= threshold:
                group.append(b["id"])
                used.add(b["id"])
        if len(group) > 1:
            used.add(a["id"])
            groups.append(group)

    return groups
