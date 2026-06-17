# Keying

Chroma key, luma key, spill suppression, and background removal.

## Chroma Key

Removes pixels based on their color (hue/saturation). The standard green/blue screen technique.

### Color Distance

The core operation — measure how similar a pixel is to the key color.

```rust
fn color_distance(pixel: &PixelF32, key: &PixelF32) -> f32 {
    let dr = pixel.red - key.red;
    let dg = pixel.green - key.green;
    let db = pixel.blue - key.blue;
    (dr * dr + dg * dg + db * db).sqrt()
}
```

### Keying Algorithm

1. Calculate distance from key color for each pixel
2. Map distance to alpha: close = transparent, far = opaque
3. Soft matte transition at edges

```rust
fn chroma_key(pixel: &PixelF32, key: &PixelF32, tolerance: f32, softness: f32) -> PixelF32 {
    let d = color_distance(pixel, key);
    let alpha = if d < tolerance {
        0.0
    } else if d < tolerance + softness {
        (d - tolerance) / softness
    } else {
        1.0
    };
    PixelF32 { red: pixel.red, green: pixel.green, blue: pixel.blue, alpha }
}
```

### HSV-based Keying

More robust — key on hue, with separate tolerance for saturation and value.

```rust
fn chroma_key_hsv(pixel: &PixelF32, key_hue: f32, hue_tol: f32,
                  sat_min: f32, val_min: f32) -> f32 {
    let (h, s, v) = rgb_to_hsv(pixel.red, pixel.green, pixel.blue);
    let hue_dist = (h - key_hue).abs().min(360.0 - (h - key_hue).abs());
    if hue_dist < hue_tol && s > sat_min && v > val_min {
        0.0 // transparent
    } else {
        1.0 // opaque
    }
}
```

## Luma Key

Removes pixels based on brightness.

```rust
fn luma_key(pixel: &PixelF32, threshold: f32, softness: f32, key_above: bool) -> f32 {
    let luma = 0.299 * pixel.red + 0.587 * pixel.green + 0.114 * pixel.blue;
    let alpha = if key_above {
        if luma > threshold + softness { 0.0 }
        else if luma > threshold { (threshold + softness - luma) / softness }
        else { 1.0 }
    } else {
        if luma < threshold - softness { 0.0 }
        else if luma < threshold { (luma - (threshold - softness)) / softness }
        else { 1.0 }
    };
    PixelF32 { red: pixel.red, green: pixel.green, blue: pixel.blue, alpha }
}
```

## Spill Suppression

After keying, the foreground often has color spill from the green/blue screen. Spill suppression desaturates or shifts the spill color.

```rust
fn suppress_spill(pixel: &PixelF32, spill_color: &PixelF32, amount: f32) -> PixelF32 {
    // Reduce the spill color channel
    let spill_luma = spill_color.red * pixel.red + spill_color.green * pixel.green + spill_color.blue * pixel.blue;
    let factor = 1.0 - amount * spill_luma;
    PixelF32 {
        red: (pixel.red * factor).max(0.0),
        green: (pixel.green * factor).max(0.0),
        blue: (pixel.blue * factor).max(0.0),
        alpha: pixel.alpha,
    }
}
```

## Edge Refinement

Key edges are the hardest part. Techniques:

- **Edge blur:** Blur the matte edge to soften transition
- **Choke/Spread:** Shrink or expand the matte
- **Color correction:** Adjust edge pixels to match foreground

```rust
fn edge_choke(matte: &[f32], w: usize, h: usize, radius: isize) -> Vec<f32> {
    let mut result = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            // Find minimum alpha in neighborhood (choke = shrink)
            let mut min_a = 1.0f32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let sx = x as isize + dx;
                    let sy = y as isize + dy;
                    if sx >= 0 && sx < w as isize && sy >= 0 && sy < h as isize {
                        min_a = min_a.min(matte[sy as usize * w + sx as usize]);
                    }
                }
            }
            result[y * w + x] = min_a;
        }
    }
    result
}
```

## AE Integration

AE's keying effects (Keylight, Color Key) follow this pipeline:
1. Color distance calculation
2. Matte generation (soft or hard)
3. Edge refinement
4. Spill suppression
5. Composite over background

Use `ae::Parameters` to expose tolerance, softness, spill amount as user controls.
