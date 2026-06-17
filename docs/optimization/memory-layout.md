# Memory Layout (Rust)

Alignment, AoS vs SoA, zero-copy patterns, and cache-friendly data structures.

## Alignment

Data alignment affects performance and correctness.

### Why Alignment Matters

- CPU reads memory in cache-line chunks (64 bytes)
- Misaligned access may require two cache-line reads
- Some SIMD instructions require aligned data (`_mm256_load_ps` requires 32-byte alignment)

### Rust Alignment

```rust
use std::mem;

assert_eq!(mem::align_of::<f32>(), 4);
assert_eq!(mem::align_of::<PixelF32>(), 4); // 4 × f32 = 16 bytes, align 4
assert_eq!(mem::size_of::<PixelF32>(), 16);
```

### Aligning Allocations

```rust
use std::alloc::{alloc, Layout};

fn alloc_aligned(size: usize, align: usize) -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align(size, align).unwrap();
        alloc(layout)
    }
}

// Cache-line aligned (64 bytes)
let ptr = alloc_aligned(1920 * 1080 * 16, 64);

// AVX2 aligned (32 bytes)
let ptr = alloc_aligned(1920 * 1080 * 16, 32);
```

### Checking Alignment

```rust
fn is_aligned_to(ptr: *const u8, alignment: usize) -> bool {
    (ptr as usize) % alignment == 0
}
```

## AoS vs SoA

### Array of Structs (AoS)

```rust
struct Pixel { r: f32, g: f32, b: f32, a: f32 }
let pixels: Vec<Pixel> = vec![...]; // [RGBA][RGBA][RGBA]...
```

- Good: Simple code, natural abstraction
- Bad for SIMD: Each SIMD register gets parts of different pixels
- Bad for cache when processing single channels

### Structure of Arrays (SoA)

```rust
struct Image {
    r: Vec<f32>,  // [RRRRRRRR]
    g: Vec<f32>,  // [GGGGGGGG]
    b: Vec<f32>,  // [BBBBBBBB]
    a: Vec<f32>,  // [AAAAAAAA]
}
```

- Good for SIMD: Process 8 reds at once with one AVX2 load
- Good for single-channel operations
- Bad: More complex code, harder to maintain pixel locality

### When to Use Which

| Scenario | Use |
|----------|-----|
| Per-pixel transforms (all channels) | AoS is fine |
| Single-channel operations | SoA |
| SIMD processing | SoA |
| Cache-heavy algorithms (blur, convolution) | Tiled + SoA hybrid |

## Zero-Copy

### Slices

```rust
// Borrow without copying
fn process(data: &[f32]) { ... }

// No allocation — just a pointer + length
let slice: &[PixelF32] = &pixels;
```

### Sub-regions

```rust
// Process a region without copying
fn process_roi(data: &[f32], width: usize, x: usize, y: usize, w: usize, h: usize) {
    for row in 0..h {
        let start = (y + row) * width + x;
        let row_slice = &data[start..start + w]; // zero-copy sub-slice
    }
}
```

### Cow (Copy on Write)

```rust
use std::borrow::Cow;

fn get_data<'a>(cache: &'a [PixelF32], frame: u32) -> Cow<'a, [PixelF32]> {
    if frame == 0 {
        Cow::Borrowed(cache) // no copy
    } else {
        let modified = transform_frame(cache, frame);
        Cow::Owned(modified) // copy on modification
    }
}
```

## Vec Layout

### Flat Vec (Preferred)

```rust
// Flat array — row-major
let buffer: Vec<PixelF32> = vec![PixelF32::default(); width * height];

// Access: O(1)
let pixel = &buffer[y * width + x];
```

### Nested Vec (Avoid)

```rust
// Vec of Vecs — allocation overhead, pointer indirection
let buffer: Vec<Vec<PixelF32>> = vec![vec![PixelF32::default(); width]; height];

// Access: two pointer dereferences
let pixel = &buffer[y][x];
```

## String/Path Memory

```rust
// Use &str for borrowed, String for owned
fn process(name: &str) { ... } // no allocation

// Path operations
use std::path::Path;
let path = Path::new("C:/output/plugin.aex");
```

## Allocation Patterns

### Pre-allocate

```rust
// Good: allocate once
let mut buffer = Vec::with_capacity(width * height);
buffer.resize(width * height, PixelF32::default());

// Bad: reallocate in loop
for frame in 0..num_frames {
    let buffer = vec![PixelF32::default(); width * height]; // allocated every frame
}
```

### Reuse Buffers

```rust
// Allocate once, reuse across frames
let mut temp_buffer = vec![0.0f32; width * height];

for frame in 0..num_frames {
    process_frame(frame, &mut temp_buffer);
    // temp_buffer is reused — no allocation
}
```

## Performance Checklist

- [ ] Hot data aligned to 32 or 64 bytes
- [ ] Flat Vec for image buffers (not nested Vec)
- [ ] Pre-allocate with `with_capacity` / `resize`
- [ ] Use slices for zero-copy borrowing
- [ ] SoA for channel-specific SIMD operations
- [ ] Avoid reallocation in per-frame loops
- [ ] Profile with `DHAT` (rustc flag: `-Z runlib=dhat`) to find allocation hotspots
