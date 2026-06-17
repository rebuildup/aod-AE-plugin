# Distortion

Lens distortion, warping, ripple effects, and displacement maps.

## Background

Distortion effects remap pixel positions — each output pixel samples from a different location in the source image.

## Bilinear Interpolation

When the source coordinate falls between pixels, interpolate from the 4 nearest neighbors.

```rust
fn sample_bilinear(src: &[PixelF32], w: usize, h: usize, x: f32, y: f32) -> PixelF32 {
    let x0 = x.floor() as isize; let y0 = y.floor() as isize;
    let x1 = x0 + 1; let y1 = y0 + 1;
    let fx = x - x0 as f32; let fy = y - y0 as f32;

    let fetch = |sx: isize, sy: isize| -> PixelF32 {
        if sx >= 0 && sx < w as isize && sy >= 0 && sy < h as isize {
            src[sy as usize * w + sx as usize]
        } else {
            PixelF32 { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }
        }
    };

    let p00 = fetch(x0, y0); let p10 = fetch(x1, y0);
    let p01 = fetch(x0, y1); let p11 = fetch(x1, y1);

    let top = lerp_pixel(&p00, &p10, fx);
    let bot = lerp_pixel(&p01, &p11, fx);
    lerp_pixel(&top, &bot, fy)
}

fn lerp_pixel(a: &PixelF32, b: &PixelF32, t: f32) -> PixelF32 {
    let it = 1.0 - t;
    PixelF32 {
        red: a.red * it + b.red * t,
        green: a.green * it + b.green * t,
        blue: a.blue * it + b.blue * t,
        alpha: a.alpha * it + b.alpha * t,
    }
}
```

## Lens Distortion

Simulates barrel/pincushion distortion from camera lenses.

### Barrel Distortion

```rust
fn barrel_distort(x: f32, y: f32, strength: f32) -> (f32, f32) {
    let r2 = x * x + y * y;
    let k = 1.0 + strength * r2;
    (x * k, y * k)
}
```

- `strength > 0` → barrel (bulges outward)
- `strength < 0` → pincushion (pinches inward)

### Brown-Conrady Model

More accurate lens distortion model:

```
r_distorted = r * (1 + k1*r² + k2*r⁴ + k3*r⁶)
```

Where k1, k2, k3 are lens-specific distortion coefficients.

## Warp / Mesh-based Distortion

For arbitrary distortion, use a mesh grid. Each vertex defines where to sample from the source.

```rust
fn warp_with_mesh(src: &[PixelF32], dst: &mut [PixelF32],
                  w: usize, h: usize, mesh: &[(f32, f32)]) {
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let (sx, sy) = mesh[idx];
            dst[idx] = sample_bilinear(src, w, h, sx, sy);
        }
    }
}
```

## Ripple Effect

Sinusoidal displacement.

```rust
fn ripple_offset(x: f32, y: f32, center_x: f32, center_y: f32,
                 amplitude: f32, frequency: f32, phase: f32) -> (f32, f32) {
    let dx = x - center_x;
    let dy = y - center_y;
    let dist = (dx * dx + dy * dy).sqrt();
    let offset = amplitude * (frequency * dist + phase).sin();
    let angle = dy.atan2(dx);
    (x + offset * angle.cos(), y + offset * angle.sin())
}
```

## Displacement Map

Uses a separate image (the displacement map) to offset each pixel.

```rust
fn displace(src: &[PixelF32], map: &[PixelF32], w: usize, h: usize,
            scale_x: f32, scale_y: f32) -> Vec<PixelF32> {
    let mut dst = vec![PixelF32::default(); w * h];
    for y in 0..h {
        for x in 0..w {
            let m = &map[y * w + x];
            let sx = x as f32 + (m.red - 0.5) * scale_x;
            let sy = y as f32 + (m.green - 0.5) * scale_y;
            dst[y * w + x] = sample_bilinear(src, w, h, sx, sy);
        }
    }
    dst
}
```

AE's `Displacement Map` effect works exactly this way — red channel offsets X, green channel offsets Y.

## Performance

- Bilinear interpolation: 4 fetches + 3 lerps per pixel
- For real-time: consider mipmap pre-filtering for large distortions
- Cache source rows when processing sequentially (avoids redundant fetches)
