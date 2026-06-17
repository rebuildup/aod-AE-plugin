# After Effects SDK

`after-effects` crate internals, PF_orld, pixel formats, and plugin lifecycle.

## The `after-effects` Crate

Rust bindings for the Adobe After Effects SDK. Version 0.4.x.

### Key Types

```rust
use after_effects as ae;

// Pixel types
ae::Pixel8     // { red: u8, green: u8, blue: u8, alpha: u8 }
ae::Pixel16    // { red: u16, green: u16, blue: u16, alpha: u16 }
ae::PixelF32   // { red: f32, green: f32, blue: f32, alpha: f32 }

// Layer — the core image access type
ae::Layer       // wraps PF_orld, provides width()/height()/pixel access

// Parameters — typed parameter access
ae::Parameters<P> // generic over your parameter enum
```

## Plugin Lifecycle

```
1. entry()           → Register effect with define_effect!()
2. describe()        → Declare parameters, pixel formats, requirements
3. render()          → Called for each frame — main processing
4. (optional)        → persist_setup(), user_changed_param(), etc.
```

### `define_effect!` Macro

```rust
after_effects::define_effect! {
    MyEffect {
        params {
            Amount: FloatSlider { default: 50.0, min: 0.0, max: 100.0 },
            Tint: Color { default: (1.0, 0.0, 0.0) },
            Mode: Popup { default: 0, items: ["Normal", "Add", "Multiply"] },
            Invert: Checkbox { default: false },
        }
    }
}
```

### Render Callback

```rust
fn render(params: &mut ae::Parameters<MyParam>,
          input: &ae::Layer,
          output: &mut ae::Layer) -> Result<(), ae::Error> {

    let amount = params.get(MyParam::Amount)?.as_float_slider()?.value();
    let width = input.width();
    let height = input.height();

    for y in 0..height {
        for x in 0..width {
            let src = *input.as_pixel32(x, y);
            let dst = output.as_pixel32_mut(x, y);

            // Process pixel
            *dst = PixelF32 {
                red: src.red * (amount / 100.0),
                green: src.green * (amount / 100.0),
                blue: src.blue * (amount / 100.0),
                alpha: src.alpha,
            };
        }
    }

    Ok(())
}
```

## Pixel Formats

### Bit Depth

AE supports 8-bit, 16-bit, and 32-bit float per channel.

- `PF_Cmd_RENDER` receives and expects the bit depth specified in `describe()`
- 32-bit float is the internal working format — always preferred for quality
- 8-bit and 16-bit are display formats

### Premultiplied Alpha

**Always premultiplied.** AE's internal pipeline assumes premultiplied alpha.

```rust
// Reading from layer: already premultiplied
let raw = *input.as_pixel32(x, y);

// Working with straight alpha
let straight = aod_ae_utils::pixel::unpremultiply(raw);

// Back to premultiplied for output
let output = aod_ae_utils::pixel::premultiply(straight_pixel);
```

### Pixel Access

```rust
// Shared reference (read-only)
let pixel: &ae::PixelF32 = layer.as_pixel32(x, y);

// Mutable reference (write)
let pixel: &mut ae::PixelF32 = layer.as_pixel32_mut(x, y);

// Bulk access (raw buffer)
let data: &[u8] = layer.buffer();
let row_bytes: isize = layer.row_bytes();
```

**Gotcha:** `row_bytes()` may be larger than `width * sizeof(Pixel)` due to alignment padding. Always use `row_bytes()` for pointer arithmetic, not `width * 4`.

## Parameters

### Parameter Types

| Type | Rust Method | Value Type |
|------|------------|------------|
| FloatSlider | `as_float_slider()?.value()` | `f64` |
| Slider (int) | `as_slider()?.value()` | `i32` |
| Color | `as_color()?.float_value()?` | `PixelF32` |
| Popup | `as_popup()?.value()` | `i32` (0-indexed) |
| Checkbox | `as_checkbox()?.value()` | `bool` |
| Angle | `as_angle()?.value()` | `f32` (convert to f64 with `.into()`) |
| Point | `as_point()?.value()` | `(f32, f32)` |

### Gotchas

- `as_float_slider()?.value()` returns `f64`, not `Result`
- `as_popup()?.value()` returns `i32`, not `Result` — wrap in `Ok()`
- `as_checkbox()?.value()` returns `bool`, not `Result` — wrap in `Ok()`
- `as_angle()?.value()` returns `f32` — use `.into()` to convert to `f64`
- `as_color()?.float_value()` returns `Result<PixelF32, Error>` — needs `?`

## Caching

AE caches render results. To enable caching:

1. Create a deterministic cache key from parameters
2. Return the same key for identical parameter states
3. AE will skip re-rendering if the key hasn't changed

```rust
use aod_ae_utils::ae_helpers::create_cache_key;

// In your effect:
let param_hash = params.hash(); // if supported
let cache_key = create_cache_key(current_time, param_hash);
```

## Error Handling

```rust
use after_effects::Error;

fn render(...) -> Result<(), Error> {
    // Param access
    let amount = params.get(MyParam::Amount)?.as_float_slider()?.value();

    // Layer fill
    output.fill(None)?;

    Ok(())
}
```

AE's error types: `Error::Generic` (string), `Error::InvalidIndex`, etc.

## Threading

- AE calls `render()` from a single thread per frame
- For multi-threaded processing, use `rayon` inside `render()`
- Don't use global mutable state — AE may call render for multiple instances concurrently
