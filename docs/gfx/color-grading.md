# Color Grading

LUT, CDL, tone mapping, ACES, and color correction pipelines.

## LUT (Look-Up Table)

A LUT maps input color values to output values. Precomputed color transform.

### 1D LUT

Maps each channel independently. Simple, fast.

```rust
fn apply_1d_lut(value: f32, lut: &[f32]) -> f32 {
    let n = lut.len() - 1;
    let x = (value * n as f32).clamp(0.0, n as f32);
    let i = x.floor() as usize;
    let f = x - x.floor();
    if i >= n { lut[n] } else { lut[i] * (1.0 - f) + lut[i + 1] * f }
}
```

### 3D LUT

Maps (R, G, B) → (R', G', B') using a 3D grid. Captures cross-channel interactions (e.g., pushing shadows toward blue affects all channels together).

```rust
fn sample_3d_lut(r: f32, g: f32, b: f32, lut: &[[[PixelF32; N]; N]; N]) -> PixelF32 {
    // Trilinear interpolation in 3D grid
    let x = r * (N - 1) as f32;
    let y = g * (N - 1) as f32;
    let z = b * (N - 1) as f32;
    // ... trilinear interpolation ...
    todo!()
}
```

Common LUT sizes: 17³, 33³, 65³. Larger = more accurate but more memory/compute.

### AE LUT Formats

AE supports .cube and .3dl LUT formats. Parse the file, then sample with trilinear interpolation.

## CDL (Color Decision List)

SOP (Slope/Offset/Power) + Sat:

```
output = clamp(((input * slope + offset) ^ power) * sat, 0, 1)
```

```rust
fn apply_cdl(pixel: &PixelF32, slope: [f32; 3], offset: [f32; 3],
             power: [f32; 3], sat: f32) -> PixelF32 {
    let r = (pixel.red * slope[0] + offset[0]).max(0.0).powf(power[0]);
    let g = (pixel.green * slope[1] + offset[1]).max(0.0).powf(power[1]);
    let b = (pixel.blue * slope[2] + offset[2]).max(0.0).powf(power[2]);
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    PixelF32 {
        red: (luma + (r - luma) * sat).clamp(0.0, 1.0),
        green: (luma + (g - luma) * sat).clamp(0.0, 1.0),
        blue: (luma + (b - luma) * sat).clamp(0.0, 1.0),
        alpha: pixel.alpha,
    }
}
```

## Tone Mapping

Maps HDR (high dynamic range) to LDR (display range 0–1).

### Reinhard

```rust
fn reinhard_tonemap(color: PixelF32) -> PixelF32 {
    let exposure = 1.0;
    let mapped = |c: f32| (c * exposure) / (1.0 + c * exposure);
    PixelF32 {
        red: mapped(color.red),
        green: mapped(color.green),
        blue: mapped(color.blue),
        alpha: color.alpha,
    }
}
```

### ACES Filmic

Industry standard (used in film/games).

```rust
fn aces_filmic(x: f32) -> f32 {
    let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
    ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0)
}
```

### ACES (full pipeline)

```rust
fn aces_tonemap(pixel: &PixelF32) -> PixelF32 {
    // sRGB → Linear
    let r = srgb_to_linear(pixel.red);
    let g = srgb_to_linear(pixel.green);
    let b = srgb_to_linear(pixel.blue);

    // Linear sRGB → ACES AP0 (matrix multiply)
    let ar = 0.59719 * r + 0.35458 * g + 0.04823 * b;
    let ag = 0.07600 * r + 0.90834 * g + 0.11566 * b;
    let ab = 0.02840 * r + 0.13383 * g + 0.83777 * b;

    // RRT + ODT fit
    let ar = aces_filmic(ar);
    let ag = aces_filmic(ag);
    let ab = aces_filmic(ab);

    // ACES AP0 → sRGB (matrix multiply)
    let r = linear_to_srgb(1.60475 * ar - 0.53108 * ag - 0.07367 * ab);
    let g = linear_to_srgb(-0.10208 * ar + 1.10813 * ag - 0.00605 * ab);
    let b = linear_to_srgb(-0.00327 * ar - 0.07276 * ag + 1.07602 * ab);

    PixelF32 { red: r, green: g, blue: b, alpha: pixel.alpha }
}
```

## Exposure / Lift/Gamma/Gain

```rust
fn exposure(pixel: &PixelF32, stops: f32) -> PixelF32 {
    let factor = 2.0f32.powf(stops);
    PixelF32 {
        red: (pixel.red * factor).clamp(0.0, 1.0),
        green: (pixel.green * factor).clamp(0.0, 1.0),
        blue: (pixel.blue * factor).clamp(0.0, 1.0),
        alpha: pixel.alpha,
    }
}

fn lift_gamma_gain(pixel: &PixelF32, lift: f32, gamma: f32, gain: f32) -> PixelF32 {
    let apply = |c: f32| ((c + lift) * gain).powf(1.0 / gamma);
    PixelF32 {
        red: apply(pixel.red).clamp(0.0, 1.0),
        green: apply(pixel.green).clamp(0.0, 1.0),
        blue: apply(pixel.blue).clamp(0.0, 1.0),
        alpha: pixel.alpha,
    }
}
```
