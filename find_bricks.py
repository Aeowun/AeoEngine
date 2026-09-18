import numpy as np
from PIL import Image
import scipy.ndimage as ndimage

img_path = r"C:\Dev\AeoEngine\.assets\textures\brick.png"
img = Image.open(img_path)
# Let's use the luminance channel
gray = img.convert("L")
arr = np.array(gray, dtype=np.uint8)

# Let's try to threshold the image to separate bricks from grout lines.
# Bricks are usually lighter or darker. Let's see the histogram or try Otsu thresholding.
hist, bin_edges = np.histogram(arr, bins=256, range=(0, 256))
# Simple Otsu's thresholding implementation
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

print(f"Otsu threshold: {threshold}")

# Threshold the image
binary = arr > threshold
# If the grout is lighter, we might need to invert, let's look at what gives better components.
# Let's count components for both binary and ~binary
for inv in [False, True]:
    b = ~binary if inv else binary
    labeled, num_features = ndimage.label(b)
    print(f"Inverted={inv}: Found {num_features} connected components")
    
    # Filter out very small or very large components to find typical bricks
    slices = ndimage.find_objects(labeled)
    brick_dims = []
    for sl in slices:
        if sl is not None:
            dy = sl[0].stop - sl[0].start
            dx = sl[1].stop - sl[1].start
            # Typical bricks shouldn't touch the border or be too small
            if 15 < dy < 200 and 15 < dx < 200:
                brick_dims.append((dx, dy))
    
    print(f"  Valid brick candidates: {len(brick_dims)}")
    if len(brick_dims) > 0:
        print(f"  Sample brick dimensions (width x height in pixels):")
        for i, (w, h) in enumerate(brick_dims[:10]):
            print(f"    Brick {i+1}: {w} x {h} (ratio width/height = {w/h:.2f})")
