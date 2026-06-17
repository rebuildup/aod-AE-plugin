# `ae_helpers` Module Reference

Typed wrappers around After Effects SDK primitives. Reduces boilerplate and type errors.

## Types

### `LayerAccessor`

Read-only wrapper around `ae::Layer`. Provides safe pixel access.

```rust
use aod_ae_utils::ae_helpers::LayerAccessor;

fn process(layer: &ae::Layer) {
    let acc = LayerAccessor::new(layer);

    let w = acc.width();
    let h = acc.height();

    for y in 0..h {
        for x in 0..w {
            if acc.bounds_check(x, y) {
                let pixel = acc.pixel_float(x, y);
                // process pixel
            }
        }
    }
}
```

### `LayerAccessorMut`

Mutable wrapper for writing back pixel data.

```rust
use aod_ae_utils::ae_helpers::LayerAccessorMut;

fn write_output(layer: &mut ae::Layer) {
    let mut acc = LayerAccessorMut::new(layer);

    for y in 0..acc.height() {
        for x in 0..acc.width() {
            acc.set_pixel_float(x, y, ae::PixelF32 {
                red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0,
            });
        }
    }

    // Or fill with solid color
    acc.fill(Some(ae::Pixel8 { red: 0, green: 0, blue: 0, alpha: 255 })).unwrap();
}
```

## Parameter Readers

Typed parameter extraction from `ae::Parameters<P>`.

```rust
use aod_ae_utils::ae_helpers::*;

// Float slider (returns f64)
let amount: f64 = get_float_param(&params, MyParam::Amount)?;

// Integer slider (returns i32)
let iterations: i32 = get_int_param(&params, MyParam::Iterations)?;

// Color picker (returns PixelF32)
let tint: ae::PixelF32 = get_color_param(&params, MyParam::TintColor)?;

// Dropdown (returns i32, 0-indexed)
let mode: i32 = get_popup_param(&params, MyParam::BlendMode)?;

// Checkbox (returns bool)
let invert: bool = get_checkbox_param(&params, MyParam::Invert)?;

// Angle dial (returns f64, in radians)
let angle: f64 = get_angle_param(&params, MyParam::Rotation)?;

// 2D point (returns (f32, f32))
let (px, py): (f32, f32) = get_point_param(&params, MyParam::Center)?;
```

**Gotcha:** `as_float_slider()?.value()` returns `f64`. The `after-effects` crate's `FloatSliderDef` uses `f64` internally.

**Gotcha:** `as_popup()?.value()` returns `i32` directly (not `Result`). It's wrapped in `Ok()` by our helper.

**Gotcha:** `as_checkbox()?.value()` returns `bool` directly (not `Result`). It's wrapped in `Ok()` by our helper.

## Cache

```rust
use aod_ae_utils::ae_helpers::create_cache_key;

// Deterministic cache key from frame + parameter hash
let key = create_cache_key(current_frame, param_hash);
```

## Logging

```rust
use aod_ae_utils::ae_helpers::{log_error, log_info, log_debug};

log_error("Something went wrong");
log_info("Processing frame 100");
log_debug("Pixel count: 2073600");
```
