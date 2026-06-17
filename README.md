# aod-ae-utils

Utility library for Adobe After Effects plugin development in Rust.

Published on [crates.io](https://crates.io/crates/aod-ae-utils).

## IMPORTANT: Read Before Using

**This library contains domain-specific knowledge that cannot be inferred from code alone.**

Before writing any code using `aod-ae-utils`, you MUST:

1. Read `docs/INDEX.md` — the documentation index with reading paths by task
2. Read the relevant docs listed in the index for your specific task
3. Do NOT guess APIs from function names — the semantics differ from what you expect

The source code alone is insufficient. The docs contain:
- Correct parameter ranges and expected value domains
- Non-obvious gotchas (e.g., premultiplied alpha vs. straight alpha)
- AE SDK quirks that are not documented elsewhere
- Algorithm background needed to choose the right approach

## Overview

This crate provides common utilities for building After Effects plugins using the [after-effects](https://crates.io/crates/after-effects) SDK (v0.4).

### Modules

- **`pixel`** — ToPixel trait, PixelBlender, channel operations
- **`color`** — sRGB/linear, HSV/HSL/OKLCH conversions, luminance
- **`buffer`** — ImageBuffer, RegionOfInterest, AE Layer conversion
- **`gpu`** — wgpu-based GPU context, texture pool, compute pipelines (opt-in)
- **`cpu`** — rayon parallel processing, SIMD helpers, tiling (default)
- **`ae_helpers`** — LayerAccessor, typed parameter readers, cache utilities

## Usage

Add to your plugin's `Cargo.toml`:

```toml
[dependencies]
after-effects = "0.4"
aod-ae-utils = "0.1"

# Optional features
aod-ae-utils = { version = "0.1", features = ["cpu"] }         # default
aod-ae-utils = { version = "0.1", features = ["full"] }        # cpu + gpu
aod-ae-utils = { version = "0.1", features = ["gpu"] }         # gpu only
```

### Example

```rust
use aod_ae_utils::color::{srgb_to_linear, linear_to_srgb, rgb_to_oklch};
use aod_ae_utils::pixel::{ToPixel, PixelBlender};
use after_effects as ae;

// Convert pixel formats
let px8 = ae::Pixel8 { red: 255, green: 128, blue: 0, alpha: 255 };
let px32: ae::PixelF32 = px8.to_pixel32();

// Color space conversion
let (l, c, h) = rgb_to_oklch(0.5, 0.3, 0.8);
let (r, g, b) = aod_ae_utils::color::oklch_to_rgb(l, c, h);
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `cpu` | Yes | rayon parallel processing, SIMD helpers |
| `gpu` | No | wgpu compute shaders, texture management |
| `full` | No | All features enabled |

## License

MPL-2.0
