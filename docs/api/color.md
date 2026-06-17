# `color` Module Reference

Color space conversions, transfer functions, and color utilities.

## Color Spaces

### sRGB ↔ Linear

```rust
use aod_ae_utils::color::{srgb_to_linear, linear_to_srgb};

// sRGB (gamma-encoded) → Linear (scene-referred)
let lin = srgb_to_linear(0.5);

// Linear → sRGB
let srgb = linear_to_srgb(0.5);
```

**Transfer function:** Standard sRGB piecewise (gamma 2.4 with linear toe). NOT a simple gamma 2.2.

**Gotcha:** `srgb_to_linear(0.5)` ≠ 0.5^2.2. The sRGB curve has a linear segment near zero. Use this function, not `powf(2.2)`.

### RGB ↔ HSV

```rust
use aod_ae_utils::color::{rgb_to_hsv, hsv_to_rgb};

let (h, s, v) = rgb_to_hsv(1.0, 0.5, 0.0); // (30.0, 1.0, 1.0)
let (r, g, b) = hsv_to_rgb(30.0, 1.0, 1.0); // (1.0, 0.5, 0.0)
```

- H: 0–360 degrees
- S: 0.0–1.0
- V: 0.0–1.0

### RGB ↔ HSL

```rust
use aod_ae_utils::color::{rgb_to_hsl, hsl_to_rgb};

let (h, s, l) = rgb_to_hsl(1.0, 0.5, 0.0);
```

- L is perceptual lightness (0.5 = mid-gray when S=0)

### RGB ↔ OKLCH

```rust
use aod_ae_utils::color::{rgb_to_oklch, oklch_to_rgb};

let (l, c, h) = oklch_to_rgb(0.6, 0.05, 30.0); // → (r, g, b) in sRGB
let (l2, c2, h2) = rgb_to_oklch(r, g, b);       // roundtrip
```

- L: 0–1 (perceptual lightness)
- C: 0–~0.4 (chroma, device-dependent)
- H: 0–360 (hue angle)

**Gotcha:** OKLCH roundtrip is only accurate for in-gamut colors. Out-of-gamut values in `oklch_to_rgb` may produce negative linear values — `linear_to_srgb` clamps these to 0, losing information.

### Gamma utilities

```rust
let gamma_corrected = apply_gamma(value, 2.2);
let linear = remove_gamma(gamma_corrected, 2.2);
```

## Luminance

```rust
use aod_ae_utils::color::{luminance_709, luminance_2020, perceived_lightness};

// Rec.709 (SDR, most AE work)
let l = luminance_709(r, g, b); // 0.2126R + 0.7152G + 0.0722B

// Rec.2020 (HDR, wide gamut)
let l = luminance_2020(r, g, b);

// CIE perceived lightness from luminance
let L = perceived_lightness(luminance); // maps 0–1 luminance to 0–100 L*
```

## Color Temperature

```rust
use aod_ae_utils::color::kelvin_to_rgb;

let (r, g, b) = kelvin_to_rgb(6500.0); // daylight white
let (r, g, b) = kelvin_to_rgb(3200.0); // tungsten warm
```

Valid range: ~1000K–40000K. Below 1000K returns black; above 40000K returns near-white.

## Utilities

```rust
use aod_ae_utils::color::*;

// Mix two colors (lerp)
let (r, g, b) = mix_color((1.0, 0.0, 0.0), (0.0, 0.0, 1.0), 0.5);

// Complementary color
let (r, g, b) = invert_color(1.0, 0.5, 0.0);

// Saturation adjustment (0.0 = grayscale, 1.0 = original, >1.0 = oversaturate)
let (r, g, b) = adjust_saturation(r, g, b, 1.2);

// Sepia tone
let (r, g, b) = sepia(r, g, b);
```
