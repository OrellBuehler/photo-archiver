import os

import cv2
import numpy as np
import torch
from PIL import Image
from simple_lama_inpainting.utils import prepare_img_and_mask, download_model

LAMA_MODEL_URL = "https://github.com/enesmsahin/simple-lama-inpainting/releases/download/v0.1.0/big-lama.pt"

_model = None
_device = None


def _get_model():
    global _model, _device
    if _model is None:
        model_path = os.environ.get("LAMA_MODEL") or download_model(LAMA_MODEL_URL)
        _device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        _model = torch.jit.load(model_path, map_location=_device)
        _model.eval()
        _model.to(_device)
    return _model, _device


def _inpaint(image: Image.Image, mask: Image.Image) -> Image.Image:
    model, device = _get_model()
    image_t, mask_t = prepare_img_and_mask(image, mask, device)
    with torch.inference_mode():
        result = model(image_t, mask_t)
        result = result[0].permute(1, 2, 0).detach().cpu().numpy()
        result = np.clip(result * 255, 0, 255).astype(np.uint8)
        return Image.fromarray(result)


def _detect_scan_lines(image: np.ndarray, window: int = 5, sensitivity: float = 2.0) -> np.ndarray:
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY).astype(np.float32)
    h, w = gray.shape
    half = window // 2

    deviations = np.zeros(h)
    for y in range(half, h - half):
        neighbors = np.delete(gray[y - half:y + half + 1], half, axis=0)
        local_median = np.median(neighbors, axis=0)
        deviations[y] = np.mean(np.abs(gray[y] - local_median))

    mean_dev = deviations[half:h - half].mean()
    std_dev = deviations[half:h - half].std()
    threshold = mean_dev + sensitivity * std_dev

    mask = np.zeros((h, w), dtype=np.uint8)
    anomalous_rows = np.where(deviations > threshold)[0]
    for row in anomalous_rows:
        mask[row, :] = 255

    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (1, 3))
    mask = cv2.dilate(mask, kernel, iterations=1)

    return mask


def remove_lines(file_path: str) -> None:
    image = cv2.imread(file_path)
    mask = _detect_scan_lines(image)

    if np.sum(mask) == 0:
        return

    pil_image = Image.fromarray(cv2.cvtColor(image, cv2.COLOR_BGR2RGB))
    pil_mask = Image.fromarray(mask)

    result = _inpaint(pil_image, pil_mask)

    result_cv = cv2.cvtColor(np.array(result), cv2.COLOR_RGB2BGR)
    cv2.imwrite(file_path, result_cv, [cv2.IMWRITE_JPEG_QUALITY, 97])
