import cv2
import numpy as np


def crop_image(file_path: str) -> None:
    img = cv2.imread(file_path)
    h, w = img.shape[:2]

    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    blurred = cv2.GaussianBlur(gray, (5, 5), 0)
    _, mask = cv2.threshold(blurred, 240, 255, cv2.THRESH_BINARY_INV)

    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (15, 15))
    mask = cv2.morphologyEx(mask, cv2.MORPH_CLOSE, kernel)

    contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    if not contours:
        return

    largest = max(contours, key=cv2.contourArea)
    x, y, cw, ch = cv2.boundingRect(largest)

    crop_area = cw * ch
    original_area = w * h
    if crop_area < original_area * 0.1:
        return
    if crop_area > original_area * 0.95:
        return

    margin = 5
    x = max(0, x - margin)
    y = max(0, y - margin)
    cw = min(w - x, cw + 2 * margin)
    ch = min(h - y, ch + 2 * margin)

    cropped = img[y:y + ch, x:x + cw]
    cv2.imwrite(file_path, cropped, [cv2.IMWRITE_JPEG_QUALITY, 97])
