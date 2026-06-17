# CPU Optimization (Rust)

rayon parallelism, SIMD, cache optimization, and memory layout in Rust.

## rayon Parallelism

rayon provides data parallelism via work-stealing thread pool.

### Parallel Iterator

```rust
use rayon::prelude::*;

// Process pixels in parallel
let output: Vec<PixelF32> = input.par_iter().map(|p| {
    transform(p)
}).collect();
```

### Parallel Chunk Processing

```rust
use rayon::prelude::*;

// Process in chunks for cache-friendly access
output.par_chunks_mut(256).enumerate().for_each(|(chunk_idx, chunk)| {
    let start_y = chunk_idx * 256 / width;
    for (i, pixel) in chunk.iter_mut().enumerate() {
        let global_idx = chunk_idx * 256 + i;
        let x = global_idx % width;
        let y = global_idx / width;
        *pixel = process(x, y);
    }
});
```

### rayon vs Manual Threading

| Approach | When to Use |
|----------|------------|
| `par_iter` | Simple per-element transforms |
| `par_chunks` | Block-based processing, needs coordinates |
| `thread::spawn` | Heterogeneous tasks, different lifetimes |
| `crossbeam` | Shared-state concurrency, channels |

**Gotcha:** rayon's work-stealing has overhead. For small images (< 100K pixels), sequential may be faster.

## SIMD (Single Instruction, Multiple Data)

### Auto-vectorization

Rust/LLVM auto-vectorizes simple loops. Write idiomatic code:

```rust
// Good — LLVM vectorizes this
for pixel in buffer.iter_mut() {
    pixel.red = pixel.red * 0.5;
}

// Bad — prevents vectorization
for i in 0..buffer.len() {
    buffer[i].red = buffer[i].red * 0.5;
    // ... complex branching ...
}
```

### Manual SIMD (std::arch)

When auto-vectorization isn't enough:

```rust
use std::arch::x86_64::*;

unsafe fn multiply_avx2(values: &mut [f32]) {
    let factor = _mm256_set1_ps(0.5);
    for chunk in values.chunks_exact_mut(8) {
        let v = _mm256_loadu_ps(chunk.as_ptr());
        let result = _mm256_mul_ps(v, factor);
        _mm256_storeu_ps(chunk.as_mut_ptr(), result);
    }
}
```

### SIMD Guidelines

- Process 8 `f32` at a time with AVX2 (256-bit registers)
- Align data to 32 bytes for `_mm256_load_ps` (faster than unaligned)
- Avoid branches inside SIMD loops — use masking instead
- For RGB pixels: process R, G, B channels separately (Structure of Arrays)

```rust
// AoS (Array of Structs) — bad for SIMD
let pixels: Vec<PixelF32> = ...;
// Each iteration loads scattered memory

// SoA (Structure of Arrays) — good for SIMD
let reds: Vec<f32> = pixels.iter().map(|p| p.red).collect();
let greens: Vec<f32> = pixels.iter().map(|p| p.green).collect();
let blues: Vec<f32> = pixels.iter().map(|p| p.blue).collect();
// Contiguous memory — SIMD loads are cache-friendly
```

## Cache Optimization

### Cache Line Size

Typically 64 bytes. Align hot data to cache line boundaries.

```rust
use std::alloc::{alloc, Layout};

unsafe fn alloc_aligned(size: usize, align: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, align).unwrap();
    alloc(layout)
}

// 64-byte aligned for cache line
let ptr = unsafe { alloc_aligned(1920 * 1080 * 4, 64) };
```

### Tiling

Process image in tiles that fit in L1 cache (~32KB).

```rust
// 32KB L1 cache / 4 bytes per f32 = 8K floats
// Tile size: 64x128 = 8192 pixels = 32KB (fits in L1)
const TILE_W: usize = 64;
const TILE_H: usize = 128;

for ty in (0..height).step_by(TILE_H) {
    for tx in (0..width).step_by(TILE_W) {
        let tw = (TILE_W).min(width - tx);
        let th = (TILE_H).min(height - ty);
        process_tile(src, dst, tx, ty, tw, th);
    }
}
```

### Loop Ordering

Access memory in the order it's stored (row-major for flat arrays).

```rust
// Good — row-major access, stride = 1
for y in 0..height {
    for x in 0..width {
        let p = &buffer[y * width + x]; // stride 1 in inner loop
    }
}

// Bad — column-major access, stride = width
for x in 0..width {
    for y in 0..height {
        let p = &buffer[y * width + x]; // stride = width, cache misses
    }
}
```

## Memory Layout

### AoS vs SoA

```rust
// AoS — simple, but bad for SIMD/cache
struct Pixel { r: f32, g: f32, b: f32, a: f32 }
let pixels: Vec<Pixel> = ...; // stride = 16 bytes

// SoA — better for SIMD, separate channels
struct Image {
    r: Vec<f32>,
    g: Vec<f32>,
    b: Vec<f32>,
    a: Vec<f32>,
}
// Each channel is contiguous — stride = 4 bytes
```

### Zero-Copy Slices

```rust
// Borrow data without copying
fn process(data: &[f32]) {
    // data is a slice — no allocation
}

// Process sub-region without copying
fn process_roi(data: &[f32], width: usize, roi: (usize, usize, usize, usize)) {
    let (x, y, w, h) = roi;
    for row in 0..h {
        let start = (y + row) * width + x;
        let slice = &data[start..start + w]; // zero-copy sub-slice
    }
}
```

## Performance Checklist

- [ ] Use `par_iter` for large buffers (>100K pixels)
- [ ] Process in cache-aligned tiles (64x128 or similar)
- [ ] Row-major access in inner loops
- [ ] Avoid bounds checks — use `get_unchecked` in hot loops (after bounds check outer)
- [ ] Use `chunks_exact` for fixed-size processing
- [ ] Align hot data to 32 bytes (AVX2) or 64 bytes (cache line)
- [ ] Profile with `perf` or `cachegrind` before optimizing
