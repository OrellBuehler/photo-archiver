import subprocess


def orient_image(file_path: str):
    """Lossless auto-rotation via exiftool."""
    try:
        subprocess.run(
            ["exiftool", "-autorot", "-overwrite_original", file_path],
            capture_output=True, text=True, timeout=30
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
