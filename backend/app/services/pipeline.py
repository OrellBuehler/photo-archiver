import asyncio
import json
import os
from app.config import settings
from app.db import get_db
from app.services.organizer import organize_image
from app.services.orienter import orient_image
from app.services.deskewer import deskew_image
from app.services.color_restorer import restore_color
from app.services.dust_remover import remove_dust
from app.services.auto_orienter import auto_orient_image
from app.utils.thumbnails import get_thumbnail_path
from app.routers.ws import manager


_processing_lock = asyncio.Lock()
_current_task_id: int | None = None


async def _broadcast(msg: dict):
    await manager.broadcast(msg)


async def run_task(task_id: int):
    global _current_task_id

    async with _processing_lock:
        _current_task_id = task_id

        try:
            async with get_db() as db:
                cursor = await db.execute("SELECT * FROM tasks WHERE id = ?", (task_id,))
                task = await cursor.fetchone()
                if not task:
                    return
                task = dict(task)

                steps = json.loads(task["steps"])

                await db.execute(
                    "UPDATE tasks SET status = 'running', started_at = datetime('now') WHERE id = ?",
                    (task_id,)
                )
                await db.commit()

            await _broadcast({"type": "task_started", "task_id": task_id})

            async with get_db() as db:
                cursor = await db.execute(
                    "SELECT ti.*, i.source_path, i.scan_id, i.year, i.month, i.organized_path "
                    "FROM task_items ti JOIN images i ON ti.image_id = i.id "
                    "WHERE ti.task_id = ? AND ti.status = 'pending' ORDER BY ti.id",
                    (task_id,)
                )
                items = [dict(r) for r in await cursor.fetchall()]

            completed = 0
            failed = 0

            for item in items:
                async with get_db() as db:
                    cursor = await db.execute("SELECT status FROM tasks WHERE id = ?", (task_id,))
                    t = await cursor.fetchone()
                    if t and dict(t)["status"] == "cancelled":
                        break

                image_id = item["image_id"]

                async with get_db() as db:
                    await db.execute(
                        "UPDATE task_items SET status = 'running', started_at = datetime('now') WHERE id = ?",
                        (item["id"],)
                    )
                    await db.commit()

                await _broadcast({
                    "type": "image_started",
                    "task_id": task_id,
                    "image_id": image_id,
                })

                try:
                    organized_path = item["organized_path"]

                    for step in steps:
                        await _broadcast({
                            "type": "step_started",
                            "task_id": task_id,
                            "image_id": image_id,
                            "step": step,
                        })

                        async with get_db() as db:
                            await db.execute(
                                "UPDATE task_items SET current_step = ? WHERE id = ?",
                                (step, item["id"])
                            )
                            await db.commit()

                        if step == "organize":
                            organized_path = await asyncio.to_thread(
                                organize_image,
                                item["source_path"],
                                item["scan_id"],
                                item["year"],
                                item["month"],
                            )
                            async with get_db() as db:
                                await db.execute(
                                    "UPDATE images SET organized_path = ?, status = 'organized', updated_at = datetime('now') WHERE id = ?",
                                    (organized_path, image_id)
                                )
                                await db.commit()

                        elif step == "orient":
                            if organized_path:
                                full_path = os.path.join(settings.output_dir, organized_path)
                                await asyncio.to_thread(orient_image, full_path)

                        elif step in ("auto_orient", "deskew", "restore_color", "remove_dust"):
                            if organized_path:
                                full_path = os.path.join(settings.output_dir, organized_path)
                            else:
                                full_path = os.path.join(settings.source_dir, item["source_path"])
                            step_fn = {"auto_orient": auto_orient_image, "deskew": deskew_image, "restore_color": restore_color, "remove_dust": remove_dust}[step]
                            await asyncio.to_thread(step_fn, full_path)

                        elif step == "enhance":
                            from app.services.enhancer import enhance_image
                            source_for_enhance = None
                            if organized_path:
                                source_for_enhance = os.path.join(settings.output_dir, organized_path)
                            else:
                                source_for_enhance = os.path.join(settings.source_dir, item["source_path"])

                            if organized_path:
                                enhanced_rel = organized_path.replace("organized/", "enhanced/", 1)
                            else:
                                scan_id = item["scan_id"] or os.path.splitext(item["source_path"])[0]
                                enhanced_rel = f"enhanced/unsorted/{scan_id}.jpg"

                            await asyncio.to_thread(
                                enhance_image, source_for_enhance, enhanced_rel
                            )
                            async with get_db() as db:
                                await db.execute(
                                    "UPDATE images SET enhanced_path = ?, status = 'enhanced', updated_at = datetime('now') WHERE id = ?",
                                    (enhanced_rel, image_id)
                                )
                                await db.commit()

                        await _broadcast({
                            "type": "step_completed",
                            "task_id": task_id,
                            "image_id": image_id,
                            "step": step,
                        })

                    thumb = get_thumbnail_path(image_id)
                    if os.path.exists(thumb):
                        os.remove(thumb)

                    async with get_db() as db:
                        await db.execute(
                            "UPDATE task_items SET status = 'completed', completed_at = datetime('now') WHERE id = ?",
                            (item["id"],)
                        )
                        await db.commit()

                    completed += 1

                except Exception as e:
                    async with get_db() as db:
                        await db.execute(
                            "UPDATE task_items SET status = 'failed', error_message = ?, completed_at = datetime('now') WHERE id = ?",
                            (str(e), item["id"])
                        )
                        await db.commit()
                    failed += 1

                async with get_db() as db:
                    await db.execute(
                        "UPDATE tasks SET completed_images = ?, failed_images = ? WHERE id = ?",
                        (completed, failed, task_id)
                    )
                    await db.commit()

                total = len(items)
                await _broadcast({
                    "type": "progress",
                    "task_id": task_id,
                    "image_id": image_id,
                    "progress": (completed + failed) / total if total else 1,
                })

            async with get_db() as db:
                cursor = await db.execute("SELECT status FROM tasks WHERE id = ?", (task_id,))
                t = await cursor.fetchone()
                final_status = "completed" if (t and dict(t)["status"] != "cancelled") else "cancelled"

                await db.execute(
                    "UPDATE tasks SET status = ?, completed_at = datetime('now') WHERE id = ?",
                    (final_status, task_id)
                )
                await db.commit()

            await _broadcast({"type": "task_completed", "task_id": task_id, "status": final_status})

        except Exception as e:
            async with get_db() as db:
                await db.execute(
                    "UPDATE tasks SET status = 'failed', error_message = ?, completed_at = datetime('now') WHERE id = ?",
                    (str(e), task_id)
                )
                await db.commit()
            await _broadcast({"type": "task_failed", "task_id": task_id, "error": str(e)})

        finally:
            _current_task_id = None
