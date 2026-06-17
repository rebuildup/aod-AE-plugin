# `pixel` Module Reference

Conversion traits and blending operations for AE pixel types.

## Types

### `ToPixel` trait

Convert between pixel formats. Implemented for `Pixel8`, `Pixel16`, `PixelF32`.

```rust
use aod_ae_utils::pixel::ToPixel;
use after_effects as ae;

let px8 = ae::Pixel8 { red: 255, green: 128, blue: 0, alpha: 255 };
let px32: ae::PixelF32 = px8.to_pixel32();
let px16: ae::Pixel16 = px8.to_pixel16();
```

**Gotcha:** `to_pixel32()` normalizes to 0.0–1.0 range. `to_pixel8()` clamps and truncates.

### `PixelBlender` trait

Apply blend modes between two pixels.

```rust
use aod_ae_utils::pixel::{ToPixel, PixelBlender};
use after_effects as ae;

let base = ae::Pixel8 { red: 100, green: 100, blue: 100, alpha: 255 };
let overlay = ae::Pixel8 { red: 200, green: 150, blue: 50, alpha: 200 };

let result = base.blend(overlay, BlendMode::Normal);
let result = base.blend(overlay, BlendMode::Multiply);
let result = base.blend(overlay, BlendMode::Screen);
```

**Available blend modes:** Normal, Multiply, Screen, Overlay, Add, Subtract, Difference, Darken, Lighten

**Gotcha:** `PixelBlender` operates on `Pixel8` only (0–255 range). For float pixels, use the functions in `color` module or implement manually.

### `PixelMap` iterator

Map over pixels in a buffer with (x, y) coordinates.

```rust
use aod_ae_utils::pixel::PixelMap;

let width = 1920;
let height = 1080;
let mut pixels = vec![ae::Pixel8::default(); width * height];

PixelMap::new(&mut pixels, width, height)
    .for_each(|(x, y, pixel)| {
        // pixel is &mut Pixel8
        pixel.red = (x as f8).min(255);
    });
```

## Functions

### Channel operations

```rust
use aod_ae_utils::pixel::*;

// Clamp value to u8 range
let v = clamp_u8(300); // 255

// Linear interpolation between channels
let v = lerp_channel(100, 200, 0.5); // 150

// Premultiply alpha
let px = premultiply(pixel); // R*A/255, G*A/255, B*A/255

// Unpremultiply alpha
let px = unpremultiply(pixel); // R*255/A, G*255/A, B*255/A
```

**Gotcha — Premultiplication:** After Effects uses premultiplied alpha internally. When reading from `PF_orld`, pixels are already premultiplied. When writing back, ensure you premultiply. The `unpremultiply` function returns 0 for fully transparent pixels (avoids division by zero).

### Grayscale / Luminance

```rust
let gray = luminance(r, g, b); // Rec.709 weights: 0.299R + 0.587G + 0.114B
let gray = to_grayscale(pixel); // converts to grayscale Pixel8
```
