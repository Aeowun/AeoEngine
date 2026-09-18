import numpy as np
from PIL import Image

img_path = r"C:\Dev\AeoEngine\.assets\textures\brick.png"
img = Image.open(img_path)
arr = np.array(img.convert("L"), dtype=np.int32)
h, w = arr.shape

# Let's count how many rows have a very low variance or distinct grout line signature
# or let's find the standard deviation along rows and columns.
print(f"Image width: {w}, height: {h}")

# Let's look for continuous lines. A grout line across the whole image would have a very consistent color or high gradient across the whole row/column.
# Let's check the average gradient of each row across the entire width:
row_grads = np.mean(np.abs(arr[1:, :] - arr[:-1, :]), axis=1)
col_grads = np.mean(np.abs(arr[:, 1:] - arr[:, :-1]), axis=0)

# Let's find prominent local maxima in row gradients and col gradients
print("\nTop 15 most prominent row gradient indices (potential horizontal grout lines):")
top_rows = np.argsort(row_grads)[::-1][:20]
print(sorted(top_rows))

print("\nTop 15 most prominent col gradient indices (potential vertical grout lines):")
top_cols = np.argsort(col_grads)[::-1][:20]
print(sorted(top_cols))

# Let's look at the actual row profiles. Let's find the rows where the image is exceptionally dark or bright (grout color)
row_means = np.mean(arr, axis=1)
col_means = np.mean(arr, axis=0)

print("\nRows with local minima/maxima of intensity:")
# Let's find rows where intensity changes sharply, or let's measure the actual brick dimensions by finding bounding boxes of connected components after thresholding, if the bricks are distinct.
# Let's print out a few slices or do a simple auto-correlation to find the exact period of bricks.
row_autocorr = np.correlate(row_means - np.mean(row_means), row_means - np.mean(row_means), mode='full')[h-1:]
col_autocorr = np.correlate(col_means - np.mean(col_means), col_means - np.mean(col_means), mode='full')[w-1:]

print(f"Row autocorrelation peaks (periodicity in height):")
for i in range(1, 100):
    if i < len(row_autocorr)-1 and row_autocorr[i] > row_autocorr[i-1] and row_autocorr[i] > row_autocorr[i+1]:
        print(f"  Lag {i}: {row_autocorr[i]:.1f}")

print(f"Col autocorrelation peaks (periodicity in width):")
for i in range(1, 100):
    if i < len(col_autocorr)-1 and col_autocorr[i] > col_autocorr[i-1] and col_autocorr[i] > col_autocorr[i+1]:
        print(f"  Lag {i}: {col_autocorr[i]:.1f}")

# Let's analyze individual bricks manually or by finding rectangles.
# Let's print out the exact coordinates where grout lines are found.
# Let's look at row_grads peaks again with a proper threshold.
import scipy.signal
try:
    r_peaks, _ = scipy.signal.find_peaks(row_grads, distance=15, prominence=2)
    c_peaks, _ = scipy.signal.find_peaks(col_grads, distance=15, prominence=2)
    print(f"\nScipy find_peaks row grout lines: {r_peaks}")
    print(f"Scipy find_peaks col grout lines: {c_peaks}")
    print(f"Row spacing (brick heights): {np.diff(r_peaks)}")
    print(f"Col spacing (brick widths): {np.diff(c_peaks)}")
except Exception as e:
    print("Scipy find_peaks failed or not used:", e)
