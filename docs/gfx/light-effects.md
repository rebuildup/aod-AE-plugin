# Light Effects

Lens flare, god rays, bloom, vignette, and light scattering.

## Lens Flare

Simulates bright light source scattering in a camera lens.

### Basic Lens Flare

```rust
fn lens_flare(x: f32, y: f32, center_x: f32, center_y: f32,
              intensity: f32, radius: f32) -> f32 {
    let dx = x - center_x;
    let dy = y - center_y;
    let dist = (dx * dx + dy * dy).sqrt();
    let falloff = 1.0 / (1.0 + dist * dist / (radius * radius));
    intensity * falloff
}
```

### Multi-element Lens Flare

Real lens flares have multiple elements (hexagonal iris, reflections). Each element is:
- Positioned along the line from light source through lens center
- Different size, color, and brightness
- Some are polygonal (iris shape)

```rust
struct FlareElement {
    offset: f32,    // position along center line (0 = center, 1 = light source)
    radius: f32,
    color: (f32, f32, f32),
    brightness: f32,
}

fn multi_flare(x: f32, y: f32, light: (f32, f32), lens_center: (f32, f32),
               elements: &[FlareElement]) -> (f32, f32, f32) {
    let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
    for elem in elements {
        // Element position: reflect through lens center
        let ex = lens_center.0 + (light.0 - lens_center.0) * elem.offset;
        let ey = lens_center.1 + (light.1 - lens_center.1) * elem.offset;
        let dx = x - ex; let dy = y - ey;
        let dist = (dx * dx + dy * dy).sqrt();
        let falloff = elem.brightness / (1.0 + dist * dist / (elem.radius * elem.radius));
        r += elem.color.0 * falloff;
        g += elem.color.1 * falloff;
        b += elem.color.2 * falloff;
    }
    (r, g, b)
}
```

## God Rays (Crepuscular Rays)

Light shafts radiating from a light source.

### Radial Blur from Light Source

```rust
fn god_rays(src: &[PixelF32], w: usize, h: usize,
            light_x: f32, light_y: f32, exposure: f32, decay: f32) -> Vec<PixelF32> {
    let samples = 64;
    let mut dst = vec![PixelF32::default(); w * h];
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - light_x;
            let dy = y as f32 - light_y;
            let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
            let mut d = 1.0f32;
            for i in 0..samples {
                let t = i as f32 / samples as f32;
                let sx = (light_x + dx * t).round() as isize;
                let sy = (light_y + dy * t).round() as isize;
                if sx >= 0 && sx < w as isize && sy >= 0 && sy < h as isize {
                    let p = &src[sy as usize * w + sx as usize];
                    r += p.red * d; g += p.green * d; b += p.blue * d;
                }
                d *= decay;
            }
            dst[y * w + x] = PixelF32 {
                red: (src[y * w + x].red + r * exposure).clamp(0.0, 1.0),
                green: (src[y * w + x].green + g * exposure).clamp(0.0, 1.0),
                blue: (src[y * w + x].blue + b * exposure).clamp(0.0, 1.0),
                alpha: 1.0,
            };
        }
    }
    dst
}
```

## Bloom

Simulates bright areas bleeding into surrounding pixels (HDR glow).

### Bloom Algorithm

1. Extract bright areas (threshold)
2. Blur the bright areas
3. Add back to original

```rust
fn bloom(src: &[PixelF32], w: usize, h: usize, threshold: f32, intensity: f32) -> Vec<PixelF32> {
    // 1. Threshold
    let bright: Vec<PixelF32> = src.iter().map(|p| {
        let luma = 0.299 * p.red + 0.587 * p.green + 0.114 * p.blue;
        if luma > threshold {
            let factor = (luma - threshold) / (1.0 - threshold);
            PixelF32 { red: p.red * factor, green: p.green * factor, blue: p.blue * factor, alpha: 1.0 }
        } else {
            PixelF32 { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }
        }
    }).collect();

    // 2. Blur (multi-pass Gaussian for large bloom)
    let blurred = multi_pass_gaussian(&bright, w, h, 4); // 4 passes

    // 3. Add back
    src.iter().zip(blurred.iter()).map(|(s, b)| PixelF32 {
        red: (s.red + b.red * intensity).clamp(0.0, 1.0),
        green: (s.green + b.green * intensity).clamp(0.0, 1.0),
        blue: (s.blue + b.blue * intensity).clamp(0.0, 1.0),
        alpha: s.alpha,
    }).collect()
}
```

## Vignette

Darkening toward edges of frame.

```rust
fn vignette(x: f32, y: f32, w: f32, h: f32, radius: f32, softness: f32) -> f32 {
    let cx = w / 2.0; let cy = h / 2.0;
    let dx = (x - cx) / cx;
    let dy = (y - cy) / cy;
    let dist = (dx * dx + dy * dy).sqrt();
    1.0 - ((dist - radius) / softness).clamp(0.0, 1.0)
}
```

## Lens Distortion Effects

### Chromatic Aberration

Offset each color channel slightly from center.

```rust
fn chromatic_aberration(x: f32, y: f32, cx: f32, cy: f32, strength: f32,
                        src: &[PixelF32], w: usize, h: usize) -> PixelF32 {
    let dx = x - cx; let dy = y - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    let offset = dist * strength / w.max(h) as f32;

    let sample_r = sample_bilinear(src, w, h, x + dx * offset, y + dy * offset);
    let sample_g = sample_bilinear(src, w, h, x, y);
    let sample_b = sample_bilinear(src, w, h, x - dx * offset, y - dy * offset);

    PixelF32 { red: sample_r.red, green: sample_g.green, blue: sample_b.blue, alpha: 1.0 }
}
```

## Performance

- God rays: O(n * samples) — expensive, consider downsampled processing
- Bloom: O(n * passes) — downsample before blur for large bloom
- Vignette: O(n) per pixel — trivial
- Lens flare: O(n * elements) — usually < 10 elements
