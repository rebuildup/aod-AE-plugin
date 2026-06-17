# `buffer` Module Reference

Image buffer management, region of interest, and AE Layer interop.

## Types

### `ImageBuffer<P>`

Generic image buffer parameterized by pixel type. Stores width, height, and pixel data in a flat `Vec<P>`.

```rust
use aod_ae_utils::buffer::ImageBuffer;
use after_effects as ae;

// Create from dimensions + closure
let buf = ImageBuffer::<ae::PixelF32>::from_fn(1920, 1080, |x, y| {
    ae::PixelF32 {
        red: x as f32 / 1920.0,
        green: y as f32 / 1080.0,
        blue: 0.0,
        alpha: 1.0,
    }
});

// Access pixels
let pixel = buf.get(100, 200); // Option<&P>
let pixel = buf.get_unchecked(100, 200); // &P (no bounds check)

// Dimensions
let w = buf.width();  // usize
let h = buf.height(); // usize

// Raw access
let slice: &[ae::PixelF32] = buf.as_slice();
let slice_mut: &mut [ae::PixelF32] = buf.as_slice_mut();
```

**Gotcha:** Pixel type `P` must be `Copy`. `ae::PixelF32` is `Copy` but does NOT implement `Default` — use `from_fn`, not `new()`.

### Type aliases

```rust
use aod_ae_utils::buffer::{Image8, Image16, ImageF32};

type Image8 = ImageBuffer<ae::Pixel8>;
type Image16 = ImageBuffer<ae::Pixel16>;
type ImageF32 = ImageBuffer<ae::PixelF32>;
```

### `RegionOfInterest`

Defines a rectangular region within an image. Used to process only a sub-region for performance.

```rust
use aod_ae_utils::buffer::RegionOfInterest;

let roi = RegionOfInterest::new(100, 100, 400, 300); // x, y, width, height
let roi = RegionOfInterest::full(1920, 1080); // entire image

// Clamp to image bounds
let safe_roi = roi.clamp_to(1920, 1080);

// Iterate over ROI coordinates
for (x, y) in roi.iter() {
    // x: usize, y: usize
}
```

## AE Layer Conversion

Convert between `ImageBuffer` and AE's `Layer` type.

### Reading from AE Layer

```rust
use aod_ae_utils::buffer::ImageBuffer;
use after_effects as ae;

fn process_layer(layer: &ae::Layer) -> ImageBuffer<ae::PixelF32> {
    let w = layer.width();
    let h = layer.height();

    ImageBuffer::<ae::PixelF32>::from_fn(w, h, |x, y| {
        *layer.as_pixel32(x, y)
    })
}
```

### Writing to AE Layer

```rust
fn write_back(buf: &ImageBuffer<ae::PixelF32>, layer: &mut ae::Layer) {
    for y in 0..buf.height() {
        for x in 0..buf.width() {
            *layer.as_pixel32_mut(x, y) = *buf.get(x, y).unwrap();
        }
    }
}
```

**Gotcha:** AE Layer dimensions are `usize`. Row stride may differ from `width * sizeof(Pixel)` — always use `layer.row_bytes()` for raw buffer access, not `width * 4`.

**Gotcha:** `layer.as_pixel32()` returns a reference. You cannot modify pixels through the shared reference — you need `as_pixel32_mut()`.
