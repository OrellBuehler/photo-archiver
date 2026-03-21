import json
import asyncio
from fastapi import APIRouter, HTTPException
from app.db import get_db
from app.models import TaskOut, TaskCreate, TaskItemOut
from app.services.pipeline import run_task, is_image_processing

router = APIRouter(prefix="/api", tags=["processing"])


@router.post("/processing/batch", response_model=TaskOut)
async def create_batch_task(req: TaskCreate):
    async with get_db() as db:
        if req.image_ids == "all":
            cursor = await db.execute("SELECT id FROM images")
            rows = await cursor.fetchall()
            image_ids = [dict(r)["id"] for r in rows]
        else:
            image_ids = req.image_ids

        if not image_ids:
            raise HTTPException(400, "No images specified")

        busy = [id for id in image_ids if is_image_processing(id)]
        if busy:
            raise HTTPException(409, f"Images already processing: {busy}")

        cursor = await db.execute(
            "INSERT INTO tasks (steps, total_images) VALUES (?, ?)",
            (json.dumps(req.steps), len(image_ids))
        )
        task_id = cursor.lastrowid

        for img_id in image_ids:
            await db.execute(
                "INSERT INTO task_items (task_id, image_id) VALUES (?, ?)",
                (task_id, img_id)
            )

        await db.commit()

        cursor = await db.execute("SELECT * FROM tasks WHERE id = ?", (task_id,))
        task = dict(await cursor.fetchone())

    asyncio.create_task(run_task(task_id))

    task["steps"] = json.loads(task["steps"])
    return TaskOut(**task)


@router.post("/processing/{image_id}")
async def create_single_task(image_id: int, req: TaskCreate):
    async with get_db() as db:
        cursor = await db.execute("SELECT id FROM images WHERE id = ?", (image_id,))
        if not await cursor.fetchone():
            raise HTTPException(404, "Image not found")

        if is_image_processing(image_id):
            raise HTTPException(409, f"Image {image_id} is already processing")

        cursor = await db.execute(
            "INSERT INTO tasks (steps, total_images) VALUES (?, 1)",
            (json.dumps(req.steps),)
        )
        task_id = cursor.lastrowid

        await db.execute(
            "INSERT INTO task_items (task_id, image_id) VALUES (?, ?)",
            (task_id, image_id)
        )
        await db.commit()

        cursor = await db.execute("SELECT * FROM tasks WHERE id = ?", (task_id,))
        task = dict(await cursor.fetchone())

    asyncio.create_task(run_task(task_id))

    task["steps"] = json.loads(task["steps"])
    return TaskOut(**task)


@router.delete("/processing/{task_id}")
async def cancel_task(task_id: int):
    async with get_db() as db:
        cursor = await db.execute("SELECT * FROM tasks WHERE id = ?", (task_id,))
        task = await cursor.fetchone()
        if not task:
            raise HTTPException(404, "Task not found")

        await db.execute(
            "UPDATE tasks SET status = 'cancelled' WHERE id = ? AND status IN ('pending', 'running')",
            (task_id,)
        )
        await db.commit()

    return {"status": "cancelled"}


@router.get("/tasks", response_model=list[TaskOut])
async def list_tasks():
    async with get_db() as db:
        cursor = await db.execute("SELECT * FROM tasks ORDER BY created_at DESC")
        rows = await cursor.fetchall()
        tasks = []
        for r in rows:
            t = dict(r)
            t["steps"] = json.loads(t["steps"])
            tasks.append(TaskOut(**t))
        return tasks


@router.get("/tasks/{task_id}", response_model=TaskOut)
async def get_task(task_id: int):
    async with get_db() as db:
        cursor = await db.execute("SELECT * FROM tasks WHERE id = ?", (task_id,))
        row = await cursor.fetchone()
        if not row:
            raise HTTPException(404, "Task not found")
        t = dict(row)
        t["steps"] = json.loads(t["steps"])
        return TaskOut(**t)
