from fastapi import APIRouter
from app.db import get_db
from app.config import settings
from app.models import AppSettings, AppSettingsUpdate

router = APIRouter(prefix="/api/settings", tags=["settings"])


@router.get("", response_model=AppSettings)
async def get_settings():
    return AppSettings(
        source_dir=settings.source_dir,
        output_dir=settings.output_dir,
        thumbnail_size=settings.thumbnail_size,
        device=settings.device,
    )


@router.patch("", response_model=AppSettings)
async def update_settings(body: AppSettingsUpdate):
    async with get_db() as db:
        if body.thumbnail_size is not None:
            await db.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                ("thumbnail_size", str(body.thumbnail_size)),
            )
            settings.thumbnail_size = body.thumbnail_size
        if body.device is not None:
            await db.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                ("device", body.device),
            )
            settings.device = body.device
        await db.commit()
    return AppSettings(
        source_dir=settings.source_dir,
        output_dir=settings.output_dir,
        thumbnail_size=settings.thumbnail_size,
        device=settings.device,
    )
