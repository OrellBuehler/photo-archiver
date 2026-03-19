import subprocess
import json


def read_exif(file_path: str) -> dict:
    try:
        result = subprocess.run(
            ["exiftool", "-json", file_path],
            capture_output=True, text=True, timeout=10
        )
        if result.returncode == 0:
            data = json.loads(result.stdout)
            return data[0] if data else {}
    except (subprocess.TimeoutExpired, FileNotFoundError, json.JSONDecodeError):
        pass
    return {}


def write_exif(file_path: str, tags: dict):
    args = ["exiftool", "-overwrite_original"]
    for key, value in tags.items():
        args.append(f"-{key}={value}")
    args.append(file_path)
    subprocess.run(args, capture_output=True, text=True, timeout=30)
