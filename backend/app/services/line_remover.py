import cv2
import numpy as np


def remove_lines(file_path: str) -> None:
    image = cv2.imread(file_path)
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
    h, w = gray.shape

    kernel_len = max(w // 4, 30)
    horizontal_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (kernel_len, 1))
    lines = cv2.morphologyEx(gray, cv2.MORPH_OPEN, horizontal_kernel)

    _, mask = cv2.threshold(lines, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    mask = cv2.bitwise_not(mask)

    dilate_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (1, 3))
    mask = cv2.dilate(mask, dilate_kernel, iterations=1)

    result = cv2.inpaint(image, mask, 3, cv2.INPAINT_TELEA)
    cv2.imwrite(file_path, result, [cv2.IMWRITE_JPEG_QUALITY, 97])
