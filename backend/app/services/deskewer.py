import cv2
import numpy as np


def deskew_image(file_path: str) -> None:
    img = cv2.imread(file_path)
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    edges = cv2.Canny(gray, 50, 150, apertureSize=3)

    lines = cv2.HoughLinesP(edges, 1, np.pi / 180, threshold=100, minLineLength=100, maxLineGap=10)
    if lines is None:
        return

    angles = []
    for line in lines:
        x1, y1, x2, y2 = line[0]
        angle = np.degrees(np.arctan2(y2 - y1, x2 - x1))
        angles.append(angle)

    median_angle = np.median(angles)

    if abs(median_angle) < 0.5:
        return

    h, w = img.shape[:2]
    cx, cy = w / 2, h / 2
    M = cv2.getRotationMatrix2D((cx, cy), median_angle, 1.0)

    cos_a = abs(M[0, 0])
    sin_a = abs(M[0, 1])
    new_w = int(h * sin_a + w * cos_a)
    new_h = int(h * cos_a + w * sin_a)
    M[0, 2] += (new_w - w) / 2
    M[1, 2] += (new_h - h) / 2

    rotated = cv2.warpAffine(img, M, (new_w, new_h), borderMode=cv2.BORDER_REPLICATE)

    # Largest axis-aligned inscribed rectangle after rotation
    angle_rad = abs(np.radians(median_angle))
    if angle_rad > np.pi / 4:
        angle_rad = np.pi / 2 - angle_rad
        inner_w = int(h * np.cos(angle_rad) - w * np.sin(angle_rad))
        inner_h = int(w * np.cos(angle_rad) - h * np.sin(angle_rad))
    else:
        inner_w = int(w * np.cos(angle_rad) - h * np.sin(angle_rad))
        inner_h = int(h * np.cos(angle_rad) - w * np.sin(angle_rad))

    inner_w = max(1, inner_w)
    inner_h = max(1, inner_h)

    rh, rw = rotated.shape[:2]
    x1 = (rw - inner_w) // 2
    y1 = (rh - inner_h) // 2
    cropped = rotated[y1:y1 + inner_h, x1:x1 + inner_w]

    cv2.imwrite(file_path, cropped, [cv2.IMWRITE_JPEG_QUALITY, 97])
