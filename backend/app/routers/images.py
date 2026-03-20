import os
import asyncio
from fastapi import APIRouter, HTTPException, Query
from fastapi.responses import FileResponse
from app.db import get_db
from app.config import settings
from app.models import ImageOut, ImageUpdate, ImageListResponse, ImageStats, BulkDeleteRequest, BulkUpdateRequest
from app.utils.thumbnails import generate_thumbnail, get_thumbnail_path

router = APIRouter(prefix="/api/images", tags=["images"])


@router.get("", response_model=ImageListResponse)
async def list_images(
    year: int | None = None,
    month: int | None = None,
    status: str | None = None,
    page: int = Query(1, ge=1),
    per_page: int = Query(50, ge=1, le=200),
):
    async with get_db() as db:
        conditions = []
        params = []

        if year is not None:
            conditions.append("year = ?")
            params.append(year)
        if month is not None:
            conditions.append("month = ?")
            params.append(month)
        if status is not None:
            conditions.append("status = ?")
            params.append(status)

        where = f"WHERE {' AND '.join(conditions)}" if conditions else ""

        cursor = await db.execute(f"SELECT COUNT(*) FROM images {where}", params)
        row = await cursor.fetchone()
        total = row[0]

        offset = (page - 1) * per_page
        cursor = await db.execute(
            f"SELECT * FROM images {where} ORDER BY year, month, scan_id LIMIT ? OFFSET ?",
            params + [per_page, offset]
        )
        rows = await cursor.fetchall()
        images = [ImageOut(**dict(r)) for r in rows]

        return ImageListResponse(images=images, total=total, page=page, per_page=per_page)


@router.get("/stats", response_model=list[ImageStats])
async def image_stats():
    async with get_db() as db:
        cursor = await db.execute(
            "SELECT year, status, COUNT(*) as count FROM images GROUP BY year, status ORDER BY year"
        )
        rows = await cursor.fetchall()
        return [ImageStats(**dict(r)) for r in rows]


@router.get("/{image_id}", response_model=ImageOut)
async def get_image(image_id: int):
    async with get_db() as db:
        cursor = await db.execute("SELECT * FROM images WHERE id = ?", (image_id,))
        row = await cursor.fetchone()
        if not row:
            raise HTTPException(404, "Image not found")
        return ImageOut(**dict(row))


@router.get("/{image_id}/file")
async def get_image_file(image_id: int, variant: str = "source"):
    async with get_db() as db:
        cursor = await db.execute("SELECT * FROM images WHERE id = ?", (image_id,))
        row = await cursor.fetchone()
        if not row:
            raise HTTPException(404, "Image not found")
        row = dict(row)

    if variant == "source":
        path = os.path.join(settings.source_dir, row["source_path"])
    elif variant == "organized":
        if not row["organized_path"]:
            raise HTTPException(404, "No organized version")
        path = os.path.join(settings.output_dir, row["organized_path"])
    elif variant == "enhanced":
        if not row["enhanced_path"]:
            raise HTTPException(404, "No enhanced version")
        path = os.path.join(settings.output_dir, row["enhanced_path"])
    else:
        raise HTTPException(400, "Invalid variant")

    if not os.path.exists(path):
        raise HTTPException(404, "File not found")

    return FileResponse(path, media_type="image/jpeg")


@router.get("/{image_id}/thumbnail")
async def get_thumbnail(image_id: int):
    async with get_db() as db:
        cursor = await db.execute("SELECT * FROM images WHERE id = ?", (image_id,))
        row = await cursor.fetchone()
        if not row:
            raise HTTPException(404, "Image not found")
        row = dict(row)

    thumb_path = get_thumbnail_path(image_id)
    if not os.path.exists(thumb_path):
        source_path = os.path.join(settings.source_dir, row["source_path"])
        if not os.path.exists(source_path):
            raise HTTPException(404, "Source file not found")
        await asyncio.to_thread(generate_thumbnail, source_path, image_id)

    return FileResponse(thumb_path, media_type="image/jpeg")


@router.patch("/{image_id}", response_model=ImageOut)
async def update_image(image_id: int, update: ImageUpdate):
    async with get_db() as db:
        cursor = await db.execute("SELECT * FROM images WHERE id = ?", (image_id,))
        row = await cursor.fetchone()
        if not row:
            raise HTTPException(404, "Image not found")

        updates = []
        params = []
        data = update.model_dump(exclude_unset=True)

        for key, value in data.items():
            updates.append(f"{key} = ?")
            params.append(value)

        if not updates:
            return ImageOut(**dict(row))

        updates.append("updated_at = datetime('now')")
        params.append(image_id)

        await db.execute(
            f"UPDATE images SET {', '.join(updates)} WHERE id = ?",
            params
        )
        await db.commit()

        cursor = await db.execute("SELECT * FROM images WHERE id = ?", (image_id,))
        row = await cursor.fetchone()
        return ImageOut(**dict(row))


@router.post("/bulk-delete")
async def bulk_delete_images(request: BulkDeleteRequest):
    async with get_db() as db:
        placeholders = ",".join("?" * len(request.image_ids))
        cursor = await db.execute(
            f"SELECT id, organized_path, enhanced_path FROM images WHERE id IN ({placeholders})",
            request.image_ids
        )
        rows = await cursor.fetchall()

        def delete_files():
            for row in rows:
                row = dict(row)
                if row["organized_path"]:
                    path = os.path.join(settings.output_dir, row["organized_path"])
                    if os.path.exists(path):
                        os.remove(path)
                if row["enhanced_path"]:
                    path = os.path.join(settings.output_dir, row["enhanced_path"])
                    if os.path.exists(path):
                        os.remove(path)
                thumb_path = os.path.join(settings.output_dir, ".thumbnails", f"{row['id']}.jpg")
                if os.path.exists(thumb_path):
                    os.remove(thumb_path)

        await asyncio.to_thread(delete_files)

        await db.execute(
            f"DELETE FROM task_items WHERE image_id IN ({placeholders})",
            request.image_ids
        )
        cursor = await db.execute(
            f"DELETE FROM images WHERE id IN ({placeholders})",
            request.image_ids
        )
        await db.commit()

        return {"deleted": cursor.rowcount}


@router.patch("/bulk")
async def bulk_update_images(request: BulkUpdateRequest):
    async with get_db() as db:
        updates = []
        params = []

        if request.year is not None:
            updates.append("year = ?")
            params.append(request.year)
        if request.month is not None:
            updates.append("month = ?")
            params.append(request.month)
        if request.title is not None:
            updates.append("title = ?")
            params.append(request.title)

        if not updates:
            return {"updated": 0}

        updates.append("updated_at = datetime('now')")
        placeholders = ",".join("?" * len(request.image_ids))
        params.extend(request.image_ids)

        cursor = await db.execute(
            f"UPDATE images SET {', '.join(updates)} WHERE id IN ({placeholders})",
            params
        )
        await db.commit()

        return {"updated": cursor.rowcount}
