import os
import numpy as np
from PIL import Image

img_path = r"C:\Dev\AeoEngine\.assets\textures\brick.png"
if not os.path.exists(img_path):
    print(f"Error: File not found at {img_path}")
    exit(1)

img = Image.open(img_path)
print(f"Image format: {img.format}, size: {img.size}, mode: {img.mode}")

# Convert to grayscale numpy array
arr = np.array(img.convert("L"), dtype=np.int32)
height, width = arr.shape

# 1. Sum of absolute differences between horizontally adjacent pixels
h_diff = np.abs(arr[:, 1:] - arr[:, :-1])
h_grad_sum = np.sum(h_diff)

# 2. Sum of absolute differences between vertically adjacent pixels
v_diff = np.abs(arr[1:, :] - arr[:-1, :])
v_grad_sum = np.sum(v_diff)

print(f"Horizontal Gradient Sum: {h_grad_sum}")
print(f"Vertical Gradient Sum: {v_grad_sum}")

if v_grad_sum > h_grad_sum * 1.2:
    print("Result: Vertical Gradient is significantly greater than Horizontal Gradient.")
    print("Dominant features (grout lines) are HORIZONTAL.")
elif h_grad_sum > v_grad_sum * 1.2:
    print("Result: Horizontal Gradient is significantly greater than Vertical Gradient.")
    print("Dominant features (grout lines) are VERTICAL.")
else:
    print("Result: Horizontal and Vertical gradients are comparable.")

# 3. Analyze brick dimensions by looking at the grout line frequencies
# Let's find rows/cols with low/high gradients or intensity changes to locate grout lines.
# Bricks usually have distinct color from grout, or grout lines show up as peaks in gradients.
row_diff_mean = np.mean(v_diff, axis=1) # average vertical change per row
col_diff_mean = np.mean(h_diff, axis=0) # average horizontal change per col

print("\n--- Profile analysis for grout lines ---")
# Find peaks in row_diff_mean and col_diff_mean to identify spacing
import scipy
from scipy.signal import find_peaks

# If scipy is not available, we can do a simple peak finding
def simple_find_peaks(arr, threshold_factor=1.5):
    peaks = []
    mean_val = np.mean(arr)
    std_val = np.std(arr)
    thresh = mean_val + 0.5 * std_val
    for i in range(1, len(arr) - 1):
        if arr[i] > arr[i-1] and arr[i] > arr[i+1] and arr[i] > thresh:
            peaks.append(i)
    return peaks

row_peaks = simple_find_peaks(row_diff_mean)
col_peaks = simple_find_peaks(col_diff_mean)

print(f"Detected grout line row indices (vertical transitions): {row_peaks[:15]}")
print(f"Detected grout line col indices (horizontal transitions): {col_peaks[:15]}")

if len(row_peaks) > 1:
    row_spacings = np.diff(row_peaks)
    print(f"Row spacings (heights of bricks/rows): {row_spacings[:10]} -> Median: {np.median(row_spacings)}")
if len(col_peaks) > 1:
    col_spacings = np.diff(col_peaks)
    print(f"Col spacings (widths of bricks/cols): {col_spacings[:10]} -> Median: {np.median(col_spacings)}")

