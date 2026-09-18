import numpy as np
from PIL import Image
import scipy.ndimage as ndimage

img_path = r"C:\Dev\AeoEngine\.assets\textures\brick.png"
img = Image.open(img_path)
gray = img.convert("L")
arr = np.array(gray, dtype=np.uint8)

# Otsu thresholding
hist, _ = np.histogram(arr, bins=256, range=(0, 256))
total = arr.size
current_max = 0
threshold = 0
sum_total = np.sum(arr)
sum_B = 0
wB = 0
for t in range(256):
    wB += hist[t]
    if wB == 0:
        continue
    wF = total - wB
    if wF == 0:
        break
    sum_B += t * hist[t]
    mB = sum_B / wB
    mF = (sum_total - sum_B) / wF
    var_between = wB * wF * (mB - mF) ** 2
    if var_between > current_max:
        current_max = var_between
        threshold = t

binary = arr > threshold
b = ~binary  # since Inverted=True gave better results

labeled, num_features = ndimage.label(b)
slices = ndimage.find_objects(labeled)

print("List of typical bricks found in the artwork:")
count = 0
widths = []
heights = []
ratios = []

for sl in slices:
    if sl is not None:
        dy = sl[0].stop - sl[0].start
        dx = sl[1].stop - sl[1].start
        # Filter for typical brick sizes based on our previous finding
        # e.g., height around 25-35, width around 50-80
        if 20 <= dy <= 40 and 40 <= dx <= 90:
            count += 1
            widths.append(dx)
            heights.append(dy)
            ratios.append(dx / dy)
            if count <= 15:
                print(f"  Brick {count}: Width = {dx} px, Height = {dy} px, Aspect Ratio = {dx/dy:.2f}")

if count > 0:
    print(f"\nSummary of {count} typical bricks:")
    print(f"  Average Width: {np.mean(widths):.1f} pixels")
    print(f"  Average Height: {np.mean(heights):.1f} pixels")
    print(f"  Average Aspect Ratio (Width / Height): {np.mean(ratios):.2f}")
else:
    print("No bricks matched the strict filtering criteria, let's relax criteria.")
