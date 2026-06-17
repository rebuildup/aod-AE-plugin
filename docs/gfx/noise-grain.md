# Noise & Grain

Procedural noise, Perlin, simplex, film grain, and denoising.

## White Noise

Random value per pixel. No coherence.

```rust
fn white_noise(seed: u32) -> f32 {
    // Simple hash-based noise
    let x = seed.wrapping_mul(73856093).wrapping_mul(19349669);
    (x as f32) / (u32::MAX as f32)
}
```

## Perlin Noise

Gradient noise — smooth, continuous. The foundation of procedural textures.

### How It Works

1. Divide space into grid cells
2. Assign random gradient vector at each grid point
3. For a point inside a cell, interpolate gradients from surrounding grid points
4. Result: smooth noise with value range approximately [-1, 1]

```rust
fn perlin_2d(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();

    let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0); // improved Perlin fade
    let u = fade(xf);
    let v = fade(yf);

    // Hash grid point coordinates to gradient
    let grad = |hash: i32, x: f32, y: f32| -> f32 {
        match hash & 3 {
            0 => x + y,
            1 => -x + y,
            2 => x - y,
            _ => -x - y,
        }
    };

    let n00 = grad(hash(xi, yi), xf, yf);
    let n10 = grad(hash(xi + 1, yi), xf - 1.0, yf);
    let n01 = grad(hash(xi, yi + 1), xf, yf - 1.0);
    let n11 = grad(hash(xi + 1, yi + 1), xf - 1.0, yf - 1.0);

    let nx0 = n00 * (1.0 - u) + n10 * u;
    let nx1 = n01 * (1.0 - u) + n11 * u;
    nx0 * (1.0 - v) + nx1 * v
}

fn hash(x: i32, y: i32) -> i32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}
```

### Fractal Brownian Motion (fBm)

Layer multiple octaves of noise for natural-looking turbulence.

```rust
fn fbm(x: f32, y: f32, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    for _ in 0..octaves {
        value += amplitude * perlin_2d(x * frequency, y * frequency);
        frequency *= lacunarity; // typically 2.0
        amplitude *= gain;       // typically 0.5
    }
    value
}
```

## Simplex Noise

Faster and more uniform than Perlin noise. Uses simplices (triangles) instead of grids.

The algorithm is similar but:
- Grid is skewed (N+1 dimensions for N-dimensional noise)
- Interpolation uses a radial falloff function
- No directional artifacts

For implementation, use the `noise` crate or follow Stefan Gustavson's reference.

## Film Grain

Simulates analog film grain (random per-pixel noise added to the image).

### Basic Grain

```rust
fn add_grain(pixel: &PixelF32, amount: f32, seed: u32) -> PixelF32 {
    let noise = (white_noise(seed) - 0.5) * amount;
    PixelF32 {
        red: (pixel.red + noise).clamp(0.0, 1.0),
        green: (pixel.green + noise).clamp(0.0, 1.0),
        blue: (pixel.blue + noise).clamp(0.0, 1.0),
        alpha: pixel.alpha,
    }
}
```

### Perceptual Grain

Grain is more visible in midtones, less in shadows/highlights. Apply noise based on luminance.

```rust
fn perceptual_grain(pixel: &PixelF32, amount: f32, seed: u32) -> PixelF32 {
    let luma = 0.299 * pixel.red + 0.587 * pixel.green + 0.114 * pixel.blue;
    // Grain visibility follows an inverted U curve — peaks around mid-gray
    let visibility = 1.0 - (2.0 * luma - 1.0).abs(); // 0 in shadows/highlights, 1 at mid-gray
    let noise = (white_noise(seed) - 0.5) * amount * visibility;
    PixelF32 {
        red: (pixel.red + noise).clamp(0.0, 1.0),
        green: (pixel.green + noise).clamp(0.0, 1.0),
        blue: (pixel.blue + noise).clamp(0.0, 1.0),
        alpha: pixel.alpha,
    }
}
```

## Denoising

Removing unwanted noise from images.

### Box Filter (simplest)

Average over a window. Blurs everything equally.

### Bilateral Filter (edge-preserving)

Averages only similar-valued neighbors — preserves edges.

```rust
fn bilateral(src: &[PixelF32], w: usize, h: usize, radius: usize,
            spatial_sigma: f32, color_sigma: f32) -> Vec<PixelF32> {
    let mut dst = vec![PixelF32::default(); w * h];
    for y in 0..h {
        for x in 0..w {
            let center = &src[y * w + x];
            let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
            let mut weight_sum = 0.0f32;
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    let sx = x as i32 + dx; let sy = y as i32 + dy;
                    if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                        let p = &src[sy as usize * w + sx as usize];
                        let spatial = (-(dx*dx + dy*dy) as f32 / (2.0 * spatial_sigma * spatial_sigma)).exp();
                        let dr = p.red - center.red; let dg = p.green - center.green; let db = p.blue - center.blue;
                        let color = (-(dr*dr + dg*dg + db*db) / (2.0 * color_sigma * color_sigma)).exp();
                        let w = spatial * color;
                        r += p.red * w; g += p.green * w; b += p.blue * w;
                        weight_sum += w;
                    }
                }
            }
            dst[y * w + x] = PixelF32 { red: r / weight_sum, green: g / weight_sum, blue: b / weight_sum, alpha: center.alpha };
        }
    }
    dst
}
```

## AE Integration

- AE's `Fractal Noise` uses Perlin/fBm internally
- `Add Grain` uses film grain simulation
- `Remove Grain` uses temporal/spatial denoising
- All noise functions should be seeded per-frame for temporal variation
