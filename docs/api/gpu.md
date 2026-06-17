# `gpu` Module Reference

wgpu-based GPU compute helpers. Requires `gpu` or `full` feature.

## Setup

```toml
[dependencies]
aod-ae-utils = { version = "0.1", features = ["gpu"] }
```

## Types

### `GpuContext`

Initializes wgpu device and queue. Entry point for all GPU operations.

```rust
use aod_ae_utils::gpu::GpuContext;

let ctx = GpuContext::new().await?;
let device = ctx.device();
let queue = ctx.queue();
```

**Gotcha:** `GpuContext::new()` is async. In an AE plugin, you typically initialize once at plugin entry and store in a `Box<dyn Any>` global.

### `TexturePool`

Reusable texture pool to avoid repeated allocation/deallocation.

```rust
use aod_ae_utils::gpu::TexturePool;

let mut pool = TexturePool::new(wgpu::TextureFormat::Rgba8Unorm);

// Acquire a texture (reuses if possible)
let tex = pool.acquire(device, 1920, 1080);

// Return when done
pool.release(tex);

// Check pool size
let count = pool.pool_size();
```

**Gotcha:** `acquire()` may return a texture larger than requested (from pool). Always check `tex.width()` and `tex.height()` before use.

### `ComputePipelineCache`

Caches compiled compute pipelines by key.

```rust
use aod_ae_utils::gpu::ComputePipelineCache;

let mut cache = ComputePipelineCache::new();

let pipeline = cache.get_or_create(
    device,
    "blur_pipeline",          // cache key
    include_str!("blur.wgsl"), // shader source
    "main",                    // entry point
)?;
```

## Image Processing

### Upload pixel data to GPU

```rust
use aod_ae_utils::gpu::upload_pixels_to_texture;

let texture = upload_pixels_to_texture(
    device,
    queue,
    pixels,      // &[u8] RGBA data
    1920,        // width
    1080,        // height
);
```

### Create buffer from data

```rust
use aod_ae_utils::gpu::create_buffer_from_data;

let buffer = create_buffer_from_data(
    device,
    data,                              // &[u8]
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
);
```

## Workgroup Utilities

```rust
use aod_ae_utils::gpu::optimal_workgroup_size;

let (x, y, z) = optimal_workgroup_size(1920, 1080, 256);
// Returns workgroup dimensions that fit within max_workgroup_size
```
