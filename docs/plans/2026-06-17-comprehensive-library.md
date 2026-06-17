# aod-ae-utils: Comprehensive Library & Documentation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform aod-ae-utils into a comprehensive, production-ready library with full AE SDK coverage, GPU/CPU optimization patterns, and algorithm design documentation.

**Architecture:** Expand the library into multiple modules covering pixel operations, color spaces, image buffers, GPU helpers (wgpu), CPU optimization (SIMD, parallelism), and AE SDK wrappers. Create English documentation that teaches algorithm design fundamentals for AE plugin development.

**Tech Stack:** Rust, after-effects 0.4, wgpu 28, rayon, std::simd (nightly)

---

## Task 1: Restructure library modules

**Files:**
- Modify: `crates/aod-ae-utils/src/lib.rs`
- Create: `crates/aod-ae-utils/src/pixel.rs`
- Create: `crates/aod-ae-utils/src/color.rs`
- Create: `crates/aod-ae-utils/src/buffer.rs`
- Create: `crates/aod-ae-utils/src/gpu.rs`
- Create: `crates/aod-ae-utils/src/cpu.rs`
- Create: `crates/aod-ae-utils/src/ae_helpers.rs`

**Step 1: Create pixel.rs (extract from lib.rs)**

Move `ToPixel` trait and implementations to `pixel.rs`. Add:
- `PixelBlender` trait (normal, multiply, screen, overlay blend modes)
- `PixelMap` iterator adapter for efficient pixel transformation
- `ChannelExtractor` for splitting/combining RGBA channels

**Step 2: Create color.rs**

Color space conversion utilities:
- RGB ↔ HSV/HSL conversions
- RGB ↔ OKLCH (perceptual color space)
- Gamma correction (sRGB linearization)
- Color temperature / white balance
- Luminance calculations (Rec.709, Rec.2020)

**Step 3: Create buffer.rs**

Image buffer abstractions:
- `ImageBuffer<T>` generic over pixel type
- `RegionOfInterest` for partial image processing
- `Stride` aware iteration (handle row padding)
- Zero-copy view types (`BufferView`, `BufferViewMut`)
- Safe pixel access with bounds checking

**Step 4: Create gpu.rs**

GPU helpers using wgpu:
- `GpuContext` - device/queue management
- `TexturePool` - reusable texture allocation
- `ComputePipelineCache` - shader caching
- `BufferTransfer` - CPU↔GPU data transfer helpers
- `GpuImageProcessor` - batch GPU processing trait

**Step 5: Create cpu.rs**

CPU optimization helpers:
- `ParallelProcessor` - rayon-based parallel pixel processing
- `ChunkIterator` - cache-friendly chunked iteration
- `SimdOps` - SIMD-accelerated pixel math (when available)
- `TileProcessor` - tile-based processing for large images

**Step 6: Create ae_helpers.rs**

AE SDK convenience wrappers:
- `LayerAccessor` - safe layer pixel data access
- `ParamReader` - typed parameter extraction
- `ProgressBar` - render progress reporting
- `CacheKey` - effect cache management

**Step 7: Update lib.rs as re-export hub**

```rust
pub mod pixel;
pub mod color;
pub mod buffer;
pub mod gpu;
pub mod cpu;
pub mod ae_helpers;

pub use pixel::*;
pub use color::*;
pub use buffer::*;
```

---

## Task 2: Implement pixel.rs

**Files:**
- Create: `crates/aod-ae-utils/src/pixel.rs`

**Step 1: Pixel blending traits**

```rust
pub trait PixelBlender {
    fn normal(self, other: Self) -> Self;
    fn multiply(self, other: Self) -> Self;
    fn screen(self, other: Self) -> Self;
    fn overlay(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;
    fn subtract(self, other: Self) -> Self;
}
```

**Step 2: PixelMap iterator**

Efficient pixel transformation without allocation:
```rust
pub struct PixelMap<'a, I, F> {
    iter: I,
    f: F,
    _marker: PhantomData<&'a ()>,
}
```

**Step 3: Channel operations**

- `split_rgba(pixel) -> [channel; 4]`
- `combine_rgba(channels: [T; 4]) -> pixel`
- `extract_channel(buffer, channel_index) -> Vec<T>`
- `apply_channel(buffer, channel_index, f: impl Fn(T) -> T)`

---

## Task 3: Implement color.rs

**Files:**
- Create: `crates/aod-ae-utils/src/color.rs`

**Step 1: RGB ↔ HSV**

Standard algorithm with hue-preserving properties:
```rust
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32);
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32);
```

**Step 2: RGB ↔ OKLCH**

Perceptually uniform color space for color grading:
```rust
pub fn rgb_to_oklch(r: f32, g: f32, b: f32) -> (f32, f32, f32);
pub fn oklch_to_rgb(l: f32, c: f32, h: f32) -> (f32, f32, f32);
```

**Step 3: Gamma correction**

```rust
pub fn srgb_to_linear(c: f32) -> f32;
pub fn linear_to_srgb(c: f32) -> f32;
pub fn gamma_correct(c: f32, gamma: f32) -> f32;
```

**Step 4: Luminance**

```rust
pub fn luminance_709(r: f32, g: f32, b: f32) -> f32;
pub fn luminance_2020(r: f32, g: f32, b: f32) -> f32;
pub fn perceived_lightness(luminance: f32) -> f32;
```

---

## Task 4: Implement buffer.rs

**Files:**
- Create: `crates/aod-ae-utils/src/buffer.rs`

**Step 1: ImageBuffer type**

```rust
pub struct ImageBuffer<P: Pixel> {
    data: Vec<P>,
    width: usize,
    height: usize,
    stride: usize, // row stride in pixels
}

impl<P: Pixel> ImageBuffer<P> {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn from_raw(data: Vec<P>, width: usize, height: usize) -> Self;
    pub fn pixel(&self, x: usize, y: usize) -> Option<&P>;
    pub fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut P>;
    pub fn row(&self, y: usize) -> Option<&[P]>;
    pub fn region(&self, roi: &RegionOfInterest) -> BufferView<'_, P>;
}
```

**Step 2: RegionOfInterest**

```rust
pub struct RegionOfInterest {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
```

**Step 3: Zero-copy views**

```rust
pub struct BufferView<'a, P> {
    data: &'a [P],
    width: usize,
    height: usize,
    stride: usize,
}
```

---

## Task 5: Implement gpu.rs

**Files:**
- Create: `crates/aod-ae-utils/src/gpu.rs`

**Step 1: GpuContext**

```rust
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter: wgpu::Adapter,
}

impl GpuContext {
    pub async fn new() -> Result<Self, GpuError>;
    pub fn device(&self) -> &wgpu::Device;
    pub fn queue(&self) -> &wgpu::Queue;
}
```

**Step 2: TexturePool**

Reusable texture allocation to avoid repeated allocation:
```rust
pub struct TexturePool {
    textures: Vec<wgpu::Texture>,
    format: wgpu::TextureFormat,
}

impl TexturePool {
    pub fn acquire(&mut self, width: u32, height: u32) -> wgpu::Texture;
    pub fn release(&mut self, texture: wgpu::Texture);
}
```

**Step 3: BufferTransfer**

CPU ↔ GPU data transfer helpers:
```rust
pub fn upload_buffer(ctx: &GpuContext, data: &[u8]) -> wgpu::Buffer;
pub fn download_buffer(ctx: &GpuContext, buffer: &wgpu::Buffer) -> Vec<u8>;
pub fn upload_texture(ctx: &GpuContext, pixels: &[u8], width: u32, height: u32) -> wgpu::Texture;
```

---

## Task 6: Implement cpu.rs

**Files:**
- Create: `crates/aod-ae-utils/src/cpu.rs`

**Step 1: ParallelProcessor**

```rust
pub struct ParallelProcessor {
    chunk_size: usize,
}

impl ParallelProcessor {
    pub fn new(chunk_size: usize) -> Self;
    pub fn process<F>(&self, pixels: &mut [PixelF32], f: F)
    where F: Fn(&mut PixelF32) + Sync + Send;
}
```

**Step 2: ChunkIterator**

Cache-friendly iteration in L1/L2-sized chunks:
```rust
pub struct ChunkIterator<'a, T> {
    slice: &'a [T],
    chunk_size: usize,
    pos: usize,
}
```

**Step 3: TileProcessor**

Tile-based processing for large images:
```rust
pub struct TileProcessor {
    tile_size: usize,
}

impl TileProcessor {
    pub fn for_each_tile(width: usize, height: usize, f: impl Fn(RegionOfInterest));
}
```

---

## Task 7: Implement ae_helpers.rs

**Files:**
- Create: `crates/aod-ae-utils/src/ae_helpers.rs`

**Step 1: LayerAccessor**

Safe wrapper for layer pixel data:
```rust
pub struct LayerAccessor<'a> {
    world: &'a ae::PForld,
}

impl<'a> LayerAccessor<'a> {
    pub fn width(&self) -> i32;
    pub fn height(&self) -> i32;
    pub fn pixel(&self, x: i32, y: i32) -> Option<&PF_Pixel>;
    pub fn row(&self, y: i32) -> Option<&[PF_Pixel]>;
}
```

**Step 2: ParamReader**

Typed parameter extraction:
```rust
pub fn get_float(params: &ae::EffectParams, id: i32) -> Result<f64, ae::Error>;
pub fn get_int(params: &ae::EffectParams, id: i32) -> Result<i32, ae::Error>;
pub fn get_color(params: &ae::EffectParams, id: i32) -> Result<(f64, f64, f64), ae::Error>;
pub fn get_popup(params: &ae::EffectParams, id: i32) -> Result<i32, ae::Error>;
```

**Step 3: ProgressBar**

```rust
pub fn set_progress(world: &ae::PForld, current: i32, total: i32);
```

---

## Task 8: Update Cargo.toml with features

**Files:**
- Modify: `crates/aod-ae-utils/Cargo.toml`

**Step 1: Add optional dependencies**

```toml
[features]
default = ["cpu"]
gpu = ["wgpu", "bytemuck", "pollster"]
cpu = ["rayon"]

[dependencies]
after-effects = "0.4"
rayon = { version = "1", optional = true }
wgpu = { version = "28", optional = true, features = ["spirv"] }
bytemuck = { version = "1", optional = true, features = ["derive"] }
pollster = { version = "0.4", optional = true }
```

---

## Task 9: Write documentation - AE Plugin Architecture

**Files:**
- Create: `docs/01-architecture.md`

**Content:**
- AE plugin lifecycle (PiPL, EntryPoints, EffectRegistration)
- Parameter system (defs, UI, keyframes)
- Rendering pipeline (SmartFX, world access, pixel iteration)
- Threading model (multithreaded rendering)
- Cache system (sequence data, frame cache)

---

## Task 10: Write documentation - SDK Specification Coverage

**Files:**
- Create: `docs/02-sdk-spec.md`

**Content:**
- Complete PF_ParamTypes coverage
- Effect flags and outflags
- Pixel formats (8bit, 16bit, Float)
- Layer access patterns
- Mask and matte handling
- Audio support basics
- AEGP suites overview

---

## Task 11: Write documentation - GPU Optimization

**Files:**
- Create: `docs/03-gpu-optimization.md`

**Content:**
- wgpu integration patterns
- Compute shader design for image processing
- Texture upload/download optimization
- Memory management (pools, reuse)
- When to use GPU vs CPU
- AE GPU device access
- Workgroup sizing
- Buffer alignment rules

---

## Task 12: Write documentation - CPU Optimization

**Files:**
- Create: `docs/04-cpu-optimization.md`

**Content:**
- SIMD intrinsics for pixel math
- Cache-friendly memory access patterns
- Parallel iteration with rayon
- Loop unrolling strategies
- Branch prediction optimization
- Memory allocation strategies
- Zero-copy patterns

---

## Task 13: Write documentation - Algorithm Design Fundamentals

**Files:**
- Create: `docs/05-algorithm-design.md`

**Content:**
- Separable filters (horizontal + vertical passes)
- Convolution kernel design
- Distance transforms
- Voronoi/Delaunay approaches
- Gradient computation
- Edge detection (Canny, Sobel)
- Color quantization
- Morphological operations

---

## Task 14: Create example plugin

**Files:**
- Create: `examples/simple-effect/Cargo.toml`
- Create: `examples/simple-effect/build.rs`
- Create: `examples/simple-effect/src/lib.rs`

**Step 1: Cargo.toml**

```toml
[package]
name = "simple-effect"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
after-effects = "0.4"
aod-ae-utils = { path = "../../crates/aod-ae-utils" }
```

**Step 2: build.rs**

Standard AE plugin build script using pipl crate.

**Step 3: lib.rs**

Minimal effect demonstrating:
- Parameter definition
- Pixel iteration using aod-ae-utils
- Color conversion
- Progress reporting

---

## Task 15: Verify build

**Step 1: Build library**

```bash
cargo build -p aod-ae-utils --features full
```

**Step 2: Build example**

```bash
cargo build -p simple-effect
```

**Step 3: Run checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-features
cargo test --all-features
```
