# Compositing

Alpha blending, Porter-Duff operators, matte processing, and premultiplication.

## Background

Compositing is the process of combining multiple image layers into a single image. Every pixel in the final image is calculated from the source (foreground) and destination (background) pixels using a blending formula.

## Alpha Channel

The alpha channel represents opacity. In AE:
- `alpha = 0` → fully transparent
- `alpha = 255` (8-bit) or `1.0` (float) → fully opaque

### Straight vs Premultiplied Alpha

**Straight (unassociated) alpha:** RGB values are independent of alpha.
```
pixel = { R: 255, G: 0, B: 0, A: 128 }  // 50% transparent red
```

**Premultiplied (associated) alpha:** RGB values are multiplied by alpha.
```
pixel = { R: 127, G: 0, B: 0, A: 128 }  // R has been scaled by A/255
```

**AE uses premultiplied alpha internally.** When you read pixels from `ae::Layer`, they are premultiplied. When you write back, you must premultiply.

```rust
use aod_ae_utils::pixel::{premultiply, unpremultiply};

// From layer (already premultiplied)
let raw = *layer.as_pixel32(x, y);

// To work with straight alpha
let straight = unpremultiply(raw);

// Process...

// Back to premultiplied for output
let output = premultiply(processed);
```

**Gotcha:** Dividing by zero in `unpremultiply` is handled — returns black for `alpha = 0`.

## Porter-Duff Operators

The standard set of compositing operators defined by Porter and Duff (1984). Each operator defines a different way to combine source and destination pixels.

### Over (most common)

```
result = src + dst * (1 - src.alpha)
```

This is the standard "paint source over destination" operation. Transparent source pixels let the destination show through.

```rust
fn porter_duff_over(src: PixelF32, dst: PixelF32) -> PixelF32 {
    let sa = src.alpha;
    let da = dst.alpha;
    let out_a = sa + da * (1.0 - sa);
    if out_a == 0.0 { return PixelF32 { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }; }
    PixelF32 {
        red: (src.red * sa + dst.red * da * (1.0 - sa)) / out_a,
        green: (src.green * sa + dst.green * da * (1.0 - sa)) / out_a,
        blue: (src.blue * sa + dst.blue * da * (1.0 - sa)) / out_a,
        alpha: out_a,
    }
}
```

### Other Operators

| Operator | Formula | Use Case |
|----------|---------|----------|
| **In** | `src * dst.alpha` | Keep source only where destination is opaque |
| **Out** | `src * (1 - dst.alpha)` | Keep source only where destination is transparent |
| **Atop** | `src * dst.alpha + dst * (1 - src.alpha)` | Source where destination is opaque, destination elsewhere |
| **Xor** | `src * (1 - dst.alpha) + dst * (1 - src.alpha)` | Non-overlapping regions |
| **Add** | `src + dst` | Additive blending (light, particles) |

## Matte Processing

A matte is a grayscale image that controls transparency. AE supports several matte types:

### Alpha Matte
Uses the alpha channel of the matte layer to define opacity.

### Luma Matte
Uses the luminance of the matte layer — white = opaque, black = transparent.

### Inverted versions
Alpha Inverted Matte, Luma Inverted Matte — reverses the matte.

## Clamping

When compositing, values can exceed the valid range. Two strategies:

- **Clamp:** Clamp to [0, 1] — lossy, but stable
- **Super-white:** Allow values > 1.0 — preserves HDR data, requires float buffers

AE works in float (PF_PixelFloat) internally, so super-white is preserved until final render.

## Rust Implementation Notes

```rust
use aod_ae_utils::pixel::{ToPixel, PixelBlender, BlendMode};
use after_effects as ae;

// Simple alpha blend between two pixels
fn blend_alpha(fg: ae::PixelF32, bg: ae::PixelF32) -> ae::PixelF32 {
    let a = fg.alpha;
    let inv_a = 1.0 - a;
    ae::PixelF32 {
        red: fg.red * a + bg.red * inv_a,
        green: fg.green * a + bg.green * inv_a,
        blue: fg.blue * a + bg.blue * inv_a,
        alpha: a + bg.alpha * inv_a,
    }
}
```

**Performance note:** For full-frame compositing, use `rayon` parallel iteration and process in cache-aligned chunks (see `cpu` module).
