# Blur & Sharpen

Gaussian blur, motion blur, depth of field, and sharpening algorithms.

## Gaussian Blur

The most common blur. Uses a Gaussian (bell curve) kernel to average neighboring pixels.

### 1D Gaussian Kernel

```
G(x) = (1 / sqrt(2π σ²)) * exp(-x² / (2σ²))
```

Where σ (sigma) controls blur radius. Larger σ = more blur.

### 2D Separable Implementation

2D Gaussian is separable — apply 1D horizontal, then 1D vertical. Reduces O(n²) to O(n).

```rust
fn gaussian_kernel(radius: usize, sigma: f32) -> Vec<f32> {
    let size = 2 * radius + 1;
    let mut kernel = vec![0.0f32; size];
    let mut sum = 0.0;
    for i in 0..size {
        let x = i as f32 - radius as f32;
        kernel[i] = (-x * x / (2.0 * sigma * sigma)).exp();
        sum += kernel[i];
    }
    for v in &mut kernel {
        *v /= sum;
    }
    kernel
}

// Apply 1D kernel along rows
fn blur_horizontal(src: &[PixelF32], dst: &mut [PixelF32], w: usize, h: usize, kernel: &[f32]) {
    let radius = kernel.len() / 2;
    for y in 0..h {
        for x in 0..w {
            let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
            for (i, &k) in kernel.iter().enumerate() {
                let sx = (x as isize + i as isize - radius as isize).max(0).min(w as isize - 1) as usize;
                let p = &src[y * w + sx];
                r += p.red * k; g += p.green * k; b += p.blue * k;
            }
            dst[y * w + x] = PixelF32 { red: r, green: g, blue: b, alpha: 1.0 };
        }
    }
}
```

**Gotcha:** Gaussian blur is not edge-aware — it blurs across edges. For edge-preserving blur, use bilateral filter.

## Motion Blur

Simulates camera or object movement during exposure.

### Directional Motion Blur

Blurs along a direction vector (angle + length).

```rust
fn motion_blur_pixel(src: &[PixelF32], w: usize, h: usize,
                     x: usize, y: usize, angle: f32, length: f32) -> PixelF32 {
    let dx = angle.cos() * length;
    let dy = angle.sin() * length;
    let samples = (length * 2.0) as usize;
    let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
    for i in 0..samples {
        let t = i as f32 / samples as f32 - 0.5;
        let sx = (x as f32 + dx * t).round() as isize;
        let sy = (y as f32 + dy * t).round() as isize;
        if sx >= 0 && sx < w as isize && sy >= 0 && sy < h as isize {
            let p = &src[sy as usize * w + sx as usize];
            r += p.red; g += p.green; b += p.blue;
        }
    }
    let inv = 1.0 / samples as f32;
    PixelF32 { red: r * inv, green: g * inv, blue: b * inv, alpha: 1.0 }
}
```

### Directional blur (AE standard)

AE's `CC Radial Blur` and similar use this approach. The key is sampling along the motion vector and averaging.

## Depth of Field (DOF)

Simulates lens focus by blurring pixels based on their depth distance from the focal plane.

### Depth of Field Algorithm

1. For each pixel, know its depth (from depth pass or z-buffer)
2. Calculate circle of confusion (CoC) size based on depth vs focal distance
3. Blur by CoC radius

```
CoC = |depth - focalDistance| / focalDistance * aperture
```

### Bokeh

Real DOF produces bokeh shapes (hexagonal, circular) from the lens aperture shape. For artistic bokeh, scatter the blur kernel shape.

## Unsharp Mask (Sharpening)

Sharpening = original + amount * (original - blurred)

```rust
fn unsharp_mask(original: &PixelF32, blurred: &PixelF32, amount: f32) -> PixelF32 {
    PixelF32 {
        red: (original.red + amount * (original.red - blurred.red)).clamp(0.0, 1.0),
        green: (original.green + amount * (original.green - blurred.green)).clamp(0.0, 1.0),
        blue: (original.blue + amount * (original.blue - blurred.blue)).clamp(0.0, 1.0),
        alpha: original.alpha,
    }
}
```

Typical `amount` values: 0.5–2.0. Higher values create halos.

## Performance

- Gaussian: O(n * k) per pixel where k = kernel size. Use separable filter.
- Motion blur: O(n * samples). Samples can be 8–64.
- DOF: O(n * CoC_radius²). Expensive — use tile-based or mipmap-based approaches.

For large kernels, use FFT-based convolution (O(n log n)) instead of direct convolution.
