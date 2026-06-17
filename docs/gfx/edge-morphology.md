# Edge Detection & Morphology

Sobel, Canny, dilate, erode, and morphological operations.

## Edge Detection

### Sobel Operator

Computes gradient magnitude using 3x3 kernels.

```rust
const SOBEL_X: [[f32; 3]; 3] = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];
const SOBEL_Y: [[f32; 3]; 3] = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];

fn sobel(src: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut edges = vec![0.0f32; w * h];
    for y in 1..h-1 {
        for x in 1..w-1 {
            let mut gx = 0.0f32; let mut gy = 0.0f32;
            for ky in 0..3 {
                for kx in 0..3 {
                    let p = src[(y + ky - 1) * w + (x + kx - 1)];
                    gx += p * SOBEL_X[ky][kx];
                    gy += p * SOBEL_Y[ky][kx];
                }
            }
            edges[y * w + x] = (gx * gx + gy * gy).sqrt();
        }
    }
    edges
}
```

### Laplacian

Second derivative — detects edges without direction.

```rust
const LAPLACIAN: [[f32; 3]; 3] = [[0.0, 1.0, 0.0], [1.0, -4.0, 1.0], [0.0, 1.0, 0.0]];
```

### Canny Edge Detection

Multi-stage pipeline:

1. **Gaussian blur** — reduce noise
2. **Sobel** — compute gradient magnitude and direction
3. **Non-maximum suppression** — thin edges to single-pixel width
4. **Double thresholding** — strong/weak edge classification
5. **Edge tracking** — connect weak edges to strong edges (hysteresis)

```rust
fn canny(src: &[f32], w: usize, h: usize, low: f32, high: f32) -> Vec<bool> {
    // 1. Gaussian blur
    let blurred = gaussian_blur(src, w, h, 1.4);
    // 2. Sobel → magnitude + direction
    let (mag, dir) = sobel_with_direction(&blurred, w, h);
    // 3. Non-maximum suppression
    let suppressed = non_max_suppress(&mag, &dir, w, h);
    // 4. Double threshold
    threshold(&suppressed, w, h, low, high)
}
```

## Morphological Operations

Operate on binary/grayscale images using a structuring element (kernel).

### Erode

Shrink bright regions. Each output pixel = minimum in kernel neighborhood.

```rust
fn erode(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut dst = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut min = f32::MAX;
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    let sx = (x as i32 + dx).max(0).min(w as i32 - 1) as usize;
                    let sy = (y as i32 + dy).max(0).min(h as i32 - 1) as usize;
                    min = min.min(src[sy * w + sx]);
                }
            }
            dst[y * w + x] = min;
        }
    }
    dst
}
```

### Dilate

Expand bright regions. Each output pixel = maximum in kernel neighborhood.

```rust
fn dilate(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut dst = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut max = f32::MIN;
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    let sx = (x as i32 + dx).max(0).min(w as i32 - 1) as usize;
                    let sy = (y as i32 + dy).max(0).min(h as i32 - 1) as usize;
                    max = max.max(src[sy * w + sx]);
                }
            }
            dst[y * w + x] = max;
        }
    }
    dst
}
```

### Open / Close

- **Open** = erode then dilate — removes small bright spots
- **Close** = dilate then erode — fills small dark holes

### Hit-Miss Transform

Pattern matching — finds specific pixel arrangements in binary images.

Used for thinning, skeletonizing, and corner detection.

## AE Integration

- `Find Edges` → Sobel/Laplacian based
- `Simple Choker` → erode/dilate
- `Minimax` → erode or dilate based on parameter
- `Maximum` / `Minimum` → dilate / erode

## Performance

- Erode/dilate: O(n * k²) per pixel. Use separable implementation for rectangular kernels: O(n * k).
- For large radii, use integral images (summed area tables) for O(1) per pixel regardless of kernel size.
