import imagehash
from fastapi import APIRouter, Query
from app.db import get_db
from app.models import ImageOut, DuplicateGroup
from app.services.duplicates import find_duplicates, compute_phashes

router = APIRouter(prefix="/api/duplicates", tags=["duplicates"])


@router.post("/scan")
async def scan_duplicates():
    await compute_phashes()
    return {"status": "ok"}


@router.get("", response_model=list[DuplicateGroup])
async def get_duplicates(threshold: int = Query(6, ge=0, le=20)):
    groups = await find_duplicates(threshold)

    result = []
    async with get_db() as db:
        for group_ids in groups:
            placeholders = ",".join("?" * len(group_ids))
            cursor = await db.execute(
                f"SELECT * FROM images WHERE id IN ({placeholders})", group_ids
            )
            rows = await cursor.fetchall()
            imgs = [ImageOut(**dict(r)) for r in rows]

            hashes = []
            for img in imgs:
                cursor2 = await db.execute("SELECT phash FROM images WHERE id = ?", (img.id,))
                r = await cursor2.fetchone()
                if r and dict(r)["phash"]:
                    hashes.append(imagehash.hex_to_hash(dict(r)["phash"]))

            max_dist = 0
            for i, h1 in enumerate(hashes):
                for h2 in hashes[i + 1:]:
                    max_dist = max(max_dist, h1 - h2)

            result.append(DuplicateGroup(images=imgs, distance=max_dist))

    return result
