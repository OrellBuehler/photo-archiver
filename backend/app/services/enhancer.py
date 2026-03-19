import os
import cv2
import numpy as np
from app.config import settings

_upsampler = None
_face_enhancer = None


def _init_models():
    global _upsampler, _face_enhancer
    if _upsampler is not None:
        return

    from basicsr.archs.rrdbnet_arch import RRDBNet
    from realesrgan import RealESRGANer
    from gfpgan import GFPGANer

    device = settings.device

    model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)

    model_path = os.path.join(settings.output_dir, ".models", "RealESRGAN_x4plus.pth")
    os.makedirs(os.path.dirname(model_path), exist_ok=True)

    if not os.path.exists(model_path):
        from basicsr.utils.download_util import load_file_from_url
        load_file_from_url(
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth",
            model_dir=os.path.dirname(model_path),
        )

    _upsampler = RealESRGANer(
        scale=4,
        model_path=model_path,
        model=model,
        tile=400,
        tile_pad=10,
        pre_pad=0,
        half=False,
        device=device,
    )

    gfpgan_model_path = os.path.join(settings.output_dir, ".models", "GFPGANv1.4.pth")
    if not os.path.exists(gfpgan_model_path):
        from basicsr.utils.download_util import load_file_from_url
        load_file_from_url(
            "https://github.com/TencentARC/GFPGAN/releases/download/v1.3.4/GFPGANv1.4.pth",
            model_dir=os.path.dirname(gfpgan_model_path),
        )

    _face_enhancer = GFPGANer(
        model_path=gfpgan_model_path,
        upscale=2,
        arch="clean",
        channel_multiplier=2,
        bg_upsampler=_upsampler,
        device=device,
    )


def enhance_image(input_path: str, output_rel_path: str) -> str:
    """Enhance a single image. Returns relative path to enhanced image."""
    _init_models()

    img = cv2.imread(input_path, cv2.IMREAD_COLOR)
    if img is None:
        raise ValueError(f"Cannot read image: {input_path}")

    h, w = img.shape[:2]
    max_dim = 1024
    if max(h, w) > max_dim:
        scale = max_dim / max(h, w)
        img = cv2.resize(img, (int(w * scale), int(h * scale)), interpolation=cv2.INTER_AREA)

    _, _, output = _face_enhancer.enhance(img, has_aligned=False, only_center_face=False, paste_back=True)

    full_output = os.path.join(settings.output_dir, output_rel_path)
    os.makedirs(os.path.dirname(full_output), exist_ok=True)
    cv2.imwrite(full_output, output)

    try:
        import subprocess
        subprocess.run(
            ["exiftool", "-overwrite_original", "-TagsFromFile", input_path, full_output],
            capture_output=True, text=True, timeout=30
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass

    return output_rel_path
