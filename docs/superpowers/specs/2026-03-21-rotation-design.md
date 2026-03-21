# Manual Rotation + RotNet Auto-Orient

## Context

Scanned photos often have no EXIF orientation data, so the existing exiftool-based orient step does nothing useful. Users need both manual rotation controls and ML-based auto-orientation to get scanned photos right-side-up.

## Design

Two features: manual rotate buttons on the image detail page, and a RotNet-based auto-orient pipeline step.

### 1. Manual Rotation

**Backend endpoint:** `POST /api/images/{image_id}/rotate?direction=left|right`

Service: `backend/app/services/rotator.py`
- Function signature: `rotate_image(file_path: str, direction: str) -> tuple[int, int]`
- `direction` is `"left"` (90° CCW) or `"right"` (90° CW)
- Uses `cv2.rotate()` with `ROTATE_90_COUNTERCLOCKWISE` or `ROTATE_90_CLOCKWISE`
- Writes at JPEG quality 97
- Returns new (width, height)
- Let exceptions propagate

Endpoint behavior:
- Resolves the best available file: organized_path > source_path
- Calls `rotate_image` in a thread
- Updates width/height in DB
- Deletes existing thumbnail so it regenerates on next request
- Returns updated `ImageOut`

**Frontend:** Two buttons (rotate left, rotate right) below the image viewer in `ImageDetail.svelte`. After rotation, re-fetch the image data and bust the image cache by appending a timestamp to the URL.

### 2. RotNet Auto-Orient (`backend/app/services/auto_orienter.py`)

Predicts correct orientation (0°, 90°, 180°, 270°) using a RotNet model and applies rotation if needed.

- Function signature: `auto_orient_image(file_path: str) -> None`
- Uses ResNet50 from torchvision (already installed via realesrgan/gfpgan)
- Model: fine-tuned RotNet weights downloaded on first use to `.models/rotnet_resnet50.pth`
- If pre-trained RotNet weights aren't readily available, train-from-scratch approach:
  - Load a pretrained ResNet50 (ImageNet weights)
  - Replace final FC layer with 4-class output (0°, 90°, 180°, 270°)
  - At inference: load image, run through model, get predicted class
  - If prediction is not 0° (upright), rotate accordingly
- Lazy-load model on first call (same pattern as enhancer.py)
- Write at JPEG quality 97
- Let exceptions propagate

**Note on RotNet weights:** The original d4nst/RotNet uses Keras/TF. Since this project already has PyTorch via realesrgan, we'll use a PyTorch ResNet50 approach. If no pre-trained PyTorch RotNet weights are available for download, we can use the self-supervised trick: at inference time, rotate the image to all 4 orientations, run each through ImageNet-pretrained ResNet50, and pick the orientation that produces the highest max-class confidence (natural images produce more confident predictions when upright). This avoids needing custom weights entirely.

### Pipeline Integration

New step `auto_orient` in pipeline.py, between orient and deskew:

```
organize → orient → auto_orient → deskew → restore_color → remove_dust → enhance
```

Follows same pattern as existing cleanup steps:
- Falls back to source_path when organized_path is None
- Uses `asyncio.to_thread`

### Frontend Toggles

Add "Auto-Orient" toggle between Orient and Deskew in both:
- `processing/+page.svelte`
- `ProcessingPanel.svelte`

## Files Changed

- `backend/app/services/rotator.py` — new (manual rotation)
- `backend/app/services/auto_orienter.py` — new (RotNet auto-orient)
- `backend/app/routers/images.py` — add POST rotate endpoint
- `backend/app/services/pipeline.py` — add auto_orient step
- `frontend/src/lib/api.ts` — add rotateImage function
- `frontend/src/lib/components/ImageDetail.svelte` — add rotate buttons
- `frontend/src/routes/processing/+page.svelte` — add Auto-Orient toggle
- `frontend/src/lib/components/ProcessingPanel.svelte` — add Auto-Orient toggle

## Dependencies

No new dependencies. torchvision and torch are already installed via realesrgan/gfpgan. OpenCV is already available.
