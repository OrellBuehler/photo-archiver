import cv2
import numpy as np


def remove_dust(file_path: str) -> None:
    image = cv2.imread(file_path)
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)

    # Bright spots: pixels significantly above local mean
    bright = cv2.adaptiveThreshold(
        gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C, cv2.THRESH_BINARY_INV, 11, -10
    )
    contours, _ = cv2.findContours(bright, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    bright_mask = np.zeros(gray.shape, dtype=np.uint8)
    for c in contours:
        if cv2.contourArea(c) < 50:
            cv2.drawContours(bright_mask, [c], -1, 255, -1)

    # Dark spots: pixels significantly below local mean
    dark = cv2.adaptiveThreshold(
        gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C, cv2.THRESH_BINARY, 11, 10
    )
    contours, _ = cv2.findContours(dark, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    dark_mask = np.zeros(gray.shape, dtype=np.uint8)
    for c in contours:
        if cv2.contourArea(c) < 50:
            cv2.drawContours(dark_mask, [c], -1, 255, -1)

    mask = cv2.bitwise_or(bright_mask, dark_mask)
    result = cv2.inpaint(image, mask, 3, cv2.INPAINT_TELEA)
    cv2.imwrite(file_path, result, [cv2.IMWRITE_JPEG_QUALITY, 97])
