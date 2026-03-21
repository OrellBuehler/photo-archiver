# Scan Quality Improvements

## Context

This project restores old photos scanned from a flatbed scanner. The current pipeline has three steps: organize, orient, and enhance (Real-ESRGAN + GFPGAN). Scanned photos commonly have skew from misalignment, color fading/yellowing from age, and dust spots or scratches. These issues should be cleaned up before AI enhancement for better results.

## Design

Three new independent pipeline steps that slot between orient and enhance:

```
organize → orient → deskew → restore_color → remove_dust → enhance
```

Each step is independently toggleable in the processing UI.

### 1. Deskew (`backend/app/services/deskewer.py`)

Detects and corrects rotation from misaligned scans.

- Convert to grayscale, apply Canny edge detection
- Use Hough line transform to detect dominant lines
- Calculate median angle from detected lines
- Rotate image using `cv2.getRotationMatrix2D` with `BORDER_REPLICATE` to avoid black borders
- Then compute the largest axis-aligned inscribed rectangle and crop to it
- Skip correction if detected angle is below 0.5 degrees
- Write with high JPEG quality (`cv2.IMWRITE_JPEG_QUALITY, 97`) to minimize recompression loss
- Let exceptions propagate (pipeline catches per-image)
- Function signature: `deskew_image(file_path: str) -> None`

### 2. Color Restoration (`backend/app/services/color_restorer.py`)

Fixes age-related color degradation: yellowing, fading, low contrast.

- Convert to LAB color space
- Apply CLAHE on L channel (`clipLimit=2.0`, `tileGridSize=(8,8)`) for contrast
- Reduce yellow cast: shift A and B channels toward neutral based on their mean deviation; skip if mean deviation < 3 (photo doesn't need it)
- Light saturation boost (convert to HSV, scale S channel by ~1.15) to compensate for fading
- Write with high JPEG quality (`cv2.IMWRITE_JPEG_QUALITY, 97`)
- Let exceptions propagate
- Function signature: `restore_color(file_path: str) -> None`

### 3. Dust Removal (`backend/app/services/dust_remover.py`)

Removes dust spots and minor scratches from scans.

- Convert to grayscale
- Detect bright spots: `cv2.adaptiveThreshold` (Gaussian, blockSize=11) to handle varying exposure, filter contours by area (< 50px²)
- Detect dark spots: same adaptive threshold on inverted image, same size filter
- Combine into an inpainting mask
- Use `cv2.inpaint` with Telea algorithm (radius 3px)
- Write with high JPEG quality (`cv2.IMWRITE_JPEG_QUALITY, 97`)
- Let exceptions propagate
- Function signature: `remove_dust(file_path: str) -> None`

### Pipeline Integration (`backend/app/services/pipeline.py`)

Add three new step handlers in `run_task`, following the same pattern as orient:

```python
elif step == "deskew":
    if organized_path:
        full_path = os.path.join(settings.output_dir, organized_path)
        await asyncio.to_thread(deskew_image, full_path)

elif step == "restore_color":
    if organized_path:
        full_path = os.path.join(settings.output_dir, organized_path)
        await asyncio.to_thread(restore_color, full_path)

elif step == "remove_dust":
    if organized_path:
        full_path = os.path.join(settings.output_dir, organized_path)
        await asyncio.to_thread(remove_dust, full_path)
```

### Frontend

Add three new toggles to the step selection area, between orient and enhance:
- "Deskew" — straighten tilted scans
- "Restore Color" — fix fading and yellowing
- "Remove Dust" — clean dust spots and scratches

Both `processing/+page.svelte` and `ProcessingPanel.svelte` have a `steps` state object that needs updating.

### Pipeline fallback

When `organized_path` is None, fall back to `source_path` (same as the enhance step does). This allows running cleanup steps without organize.

## Files Changed

- `backend/app/services/deskewer.py` — new
- `backend/app/services/color_restorer.py` — new
- `backend/app/services/dust_remover.py` — new
- `backend/app/services/pipeline.py` — modified (add three step handlers)
- `frontend/src/routes/processing/+page.svelte` — modified (add three toggles)
- `frontend/src/lib/components/ProcessingPanel.svelte` — modified (add three toggles)

## Dependencies

No new dependencies. All operations use OpenCV (`cv2`) and NumPy, both already installed.
