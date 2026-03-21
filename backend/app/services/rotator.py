import cv2


def rotate_image(file_path: str, direction: str) -> tuple[int, int]:
    if direction not in ("left", "right"):
        raise ValueError(f"Invalid direction: {direction}")

    img = cv2.imread(file_path)
    if img is None:
        raise ValueError(f"Cannot read image: {file_path}")
    rotation = cv2.ROTATE_90_COUNTERCLOCKWISE if direction == "left" else cv2.ROTATE_90_CLOCKWISE
    img = cv2.rotate(img, rotation)
    cv2.imwrite(file_path, img, [cv2.IMWRITE_JPEG_QUALITY, 97])
    return img.shape[1], img.shape[0]
