# `cpu` Module Reference

CPU parallel processing, SIMD helpers, and memory-aligned tiling. Default feature.

## Types

### `ParallelProcessor`

Rayon-based parallel image processor. Splits work across threads automatically.

```rust
use aod_ae_utils::cpu::ParallelProcessor;
use after_effects as ae;

let processor = ParallelProcessor::new();

// Process a pixel buffer in parallel
processor.process_pixels(
    width,
    height,
    |x, y| -> ae::PixelF32 {
        // Called per-pixel, thread-safe
        ae::PixelF32 { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }
    },
    &mut output, // &mut [PixelF32]
);
```

**Gotcha:** The closure must be `Fn` (not `FnMut`) — no mutable state across threads. Use `AtomicUsize` or thread-local storage if you need accumulation.

### `ChunkIterator`

Splits a range into cache-friendly chunks for manual parallel iteration.

```rust
use aod_ae_utils::cpu::ChunkIterator;

let chunks: Vec<(usize, usize)> = ChunkIterator::new(0, 1920 * 1080, 64)
    .collect();

// Each chunk is (start, end) — sized to fit in L1 cache
for (start, end) in chunks {
    for i in start..end {
        let x = i % 1920;
        let y = i / 1920;
        // process pixel
    }
}
```

### `TileProcessor`

Splits an image into tiles for tiled processing (useful for large images or tiling effects).

```rust
use aod_ae_utils::cpu::TileProcessor;

let tiles = TileProcessor::new(1920, 1080, 256, 256);
// Returns iterator of (x, y, width, height) tiles

for (tx, ty, tw, th) in tiles {
    // Process tile at (tx..tx+tw, ty..ty+th)
}
```

## SIMD Helpers

Aligned memory and SIMD-optimized operations.

### Alignment check

```rust
use aod_ae_utils::cpu::{is_aligned_to, align_to};

assert!(is_aligned_to(ptr, 32)); // AVX2 alignment
let aligned = align_to(33, 32);  // → 64
```

### SIMD note

The `cpu` module provides the infrastructure for SIMD processing (aligned memory, cache-friendly chunking). Actual SIMD intrinsics are not exposed — use the `std::arch` module directly for hand-tuned kernels, or rely on LLVM auto-vectorization through idiomatic Rust loops.

**Recommended pattern for SIMD-friendly code:**

```rust
// Process aligned chunks — LLVM will auto-vectorize this
for pixel in buffer.chunks_exact(8) {
    // 8 pixels = one AVX2 register
    let r: [f32; 8] = pixel.iter().map(|p| p.red).collect::<Vec<_>>().try_into().unwrap();
    // ... process r ...
}
```
