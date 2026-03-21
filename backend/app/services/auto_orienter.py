import cv2
import numpy as np
import torch
import torchvision
import torchvision.transforms.functional as TF
from app.config import settings

_model = None


def _init_model():
    global _model
    if _model is not None:
        return

    m = torchvision.models.resnet50(weights=torchvision.models.ResNet50_Weights.DEFAULT)
    m.train(False)
    m.to(settings.device)
    _model = m


def auto_orient_image(file_path: str) -> None:
    _init_model()

    img_bgr = cv2.imread(file_path, cv2.IMREAD_COLOR)
    if img_bgr is None:
        raise ValueError(f"Cannot read image: {file_path}")

    img_rgb = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2RGB)
    small = cv2.resize(img_rgb, (224, 224), interpolation=cv2.INTER_AREA)

    rotations = [
        small,
        cv2.rotate(small, cv2.ROTATE_90_CLOCKWISE),
        cv2.rotate(small, cv2.ROTATE_180),
        cv2.rotate(small, cv2.ROTATE_90_COUNTERCLOCKWISE),
    ]

    mean = [0.485, 0.456, 0.406]
    std = [0.229, 0.224, 0.225]

    tensors = []
    for rot in rotations:
        t = torch.from_numpy(rot).permute(2, 0, 1).float() / 255.0
        t = TF.normalize(t, mean, std)
        tensors.append(t)

    batch = torch.stack(tensors).to(settings.device)

    with torch.no_grad():
        logits = _model(batch)
        probs = torch.softmax(logits, dim=1)
        confidences = probs.max(dim=1).values

    best_idx = int(confidences.argmax().item())

    if best_idx == 0:
        return

    rotate_map = {
        1: cv2.ROTATE_90_CLOCKWISE,
        2: cv2.ROTATE_180,
        3: cv2.ROTATE_90_COUNTERCLOCKWISE,
    }

    rotated = cv2.rotate(img_bgr, rotate_map[best_idx])
    cv2.imwrite(file_path, rotated, [cv2.IMWRITE_JPEG_QUALITY, 97])
