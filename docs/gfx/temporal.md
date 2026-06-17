# Temporal Effects

Frame interpolation, temporal blur, and optical flow.

## Frame Interpolation

Blending between frames to create smooth motion or slow motion.

### Linear Frame Blend

```rust
fn blend_frames(frame_a: &[PixelF32], frame_b: &[PixelF32], t: f32) -> Vec<PixelF32> {
    frame_a.iter().zip(frame_b.iter()).map(|(a, b)| {
        PixelF32 {
            red: a.red * (1.0 - t) + b.red * t,
            green: a.green * (1.0 - t) + b.green * t,
            blue: a.blue * (1.0 - t) + b.blue * t,
            alpha: a.alpha * (1.0 - t) + b.alpha * t,
        }
    }).collect()
}
```

AE's `Frame Blend` uses this approach. Simple but causes ghosting on fast motion.

### Motion-Compensated Interpolation

Better quality — estimate motion between frames, then warp.

1. Compute optical flow (motion vectors) between frame A and frame B
2. For intermediate frame, warp A forward and B backward using flow
3. Blend warped frames

## Optical Flow

Estimates per-pixel motion between two frames.

### Lucas-Kanade (Sparse)

Estimates motion at specific feature points.

```rust
fn lucas_kanade(frame_a: &[f32], frame_b: &[f32], w: usize, h: usize,
                points: &[(usize, usize)]) -> Vec<(f32, f32)> {
    // For each point, solve for (dx, dy) that minimizes:
    // sum over window: (I_a(x,y) - I_b(x+dx, y+dy))²
    // Using gradient images Ix, Iy and temporal difference It
    todo!() // implementation requires gradient computation
}
```

### Horn-Schunck (Dense)

Estimates dense flow field with smoothness constraint.

Minimize:
```
E = ∫∫ (Ix*u + Iy*v + It)² + α²*(|∇u|² + |∇v|²) dxdy
```

Where u,v are horizontal/vertical flow, Ix,Iy,It are spatiotemporal gradients, α is smoothness weight.

## Temporal Blur (Motion Blur from Frame Sequence)

Averages multiple sub-frame samples along the shutter interval.

```rust
fn temporal_blur(frames: &[&[PixelF32]], w: usize, h: usize) -> Vec<PixelF32> {
    let n = frames.len();
    let mut result = vec![PixelF32::default(); w * h];
    for i in 0..w * h {
        let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
        for frame in frames {
            r += frame[i].red; g += frame[i].green; b += frame[i].blue;
        }
        result[i] = PixelF32 {
            red: r / n as f32, green: g / n as f32, blue: b / n as f32, alpha: 1.0,
        };
    }
    result
}
```

## Time Remapping

Mapping from output time to source time (non-linear).

```rust
fn time_remap(t: f32, keyframes: &[(f32, f32)]) -> f32 {
    // Linear interpolation between keyframes
    // keyframes: [(source_time, output_time), ...]
    for i in 0..keyframes.len() - 1 {
        let (st0, ot0) = keyframes[i];
        let (st1, ot1) = keyframes[i + 1];
        if t >= ot0 && t <= ot1 {
            let frac = (t - ot0) / (ot1 - ot0);
            return st0 + (st1 - st0) * frac;
        }
    }
    keyframes.last().unwrap().0
}
```

## AE Integration

AE's time-related effects:
- **Time Warp:** Frame interpolation + motion blur
- **Echo:** Blends frames at different times
- **CC Wide Time:** Multi-frame blur
- **Posterize Time:** Reduces frame rate (frame skipping)

All operate on the `ae::Layer` frame accessor. Use `layer.frame(t)` to get frames at arbitrary times.
