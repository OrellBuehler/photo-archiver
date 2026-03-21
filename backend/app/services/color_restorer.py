import cv2
import numpy as np


def restore_color(file_path: str) -> None:
    img = cv2.imread(file_path)

    lab = cv2.cvtColor(img, cv2.COLOR_BGR2LAB).astype(np.float32)
    l, a, b = cv2.split(lab)

    clahe = cv2.createCLAHE(clipLimit=2.0, tileGridSize=(8, 8))
    l = clahe.apply(l.astype(np.uint8)).astype(np.float32)

    a_mean_dev = a.mean() - 128
    b_mean_dev = b.mean() - 128
    if abs(a_mean_dev) >= 3 or abs(b_mean_dev) >= 3:
        a -= a_mean_dev
        b -= b_mean_dev

    lab = cv2.merge([l, a, b])
    lab = np.clip(lab, 0, 255).astype(np.uint8)
    img = cv2.cvtColor(lab, cv2.COLOR_LAB2BGR)

    hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV).astype(np.float32)
    h, s, v = cv2.split(hsv)
    s = np.clip(s * 1.15, 0, 255)
    hsv = cv2.merge([h, s, v])
    img = cv2.cvtColor(hsv.astype(np.uint8), cv2.COLOR_HSV2BGR)

    cv2.imwrite(file_path, img, [cv2.IMWRITE_JPEG_QUALITY, 97])
