import os
from PIL import Image
from app.config import settings


def get_thumbnail_path(image_id: int) -> str:
    return os.path.join(settings.output_dir, ".thumbnails", f"{image_id}.jpg")


def generate_thumbnail(source_path: str, image_id: int) -> str:
    thumb_path = get_thumbnail_path(image_id)
    if os.path.exists(thumb_path):
        return thumb_path

    os.makedirs(os.path.dirname(thumb_path), exist_ok=True)

    with Image.open(source_path) as img:
        img.thumbnail((settings.thumbnail_size, settings.thumbnail_size))
        if img.mode != "RGB":
            img = img.convert("RGB")
        img.save(thumb_path, "JPEG", quality=85)

    return thumb_path
