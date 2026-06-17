# GPU Optimization (wgpu)

wgpu compute shaders, texture management, and buffer strategies for AE plugins.

## wgpu Architecture

wgpu is a safe, cross-platform GPU API. AE plugins use compute shaders for parallel image processing.

### Initialization

```rust
use wgpu::Instance;

let instance = Instance::new(&wgpu::InstanceDescriptor {
    backends: wgpu::Backends::all(),
    ..Default::default()
});

let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
    power_preference: wgpu::PowerPreference::HighPerformance,
    ..Default::default()
}).await.ok_or(GpuError::AdapterRequestFailed)?;

let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
    label: Some("ae-plugin"),
    ..Default::default()
}).await?;
```

**Gotcha:** wgpu 28 changed the API — `request_adapter` returns `Result`, not `Option`. See `gpu.rs` in this crate for the correct API.

## Compute Shaders

### WGSL Shader Example

```wgsl
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

struct Params {
    width: u32,
    height: u32,
    kernel_size: u32,
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&input)) { return; }

    // Process pixel
    output[idx] = input[idx] * 0.5;
}
```

### Dispatch

```rust
let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("compute shader"),
    source: wgpu::ShaderSource::Wgsl(shader_source.into()),
});

let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("compute pipeline"),
    layout: None,
    module: &shader,
    entry_point: Some("main"),
    compilation_options: Default::default(),
    cache: None,
});

let bind_group_layout = pipeline.get_bind_group_layout(0);
let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("bind group"),
    layout: &bind_group_layout,
    entries: &[
        wgpu::BindGroupEntry { binding: 0, resource: input_buffer.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 1, resource: output_buffer.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 2, resource: params_buffer.as_entire_binding() },
    ],
});

let mut encoder = device.create_command_encoder(&Default::default());
{
    let mut pass = encoder.begin_compute_pass(&Default::default());
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups((num_elements as u32 + 255) / 256, 1, 1);
}
queue.submit(Some(encoder.finish()));
```

## Buffer Management

### Buffer Types

| Buffer | Usage | Speed |
|--------|-------|-------|
| `STORAGE` | Compute shader read/write | GPU-only |
| `COPY_SRC` | GPU → CPU copy | — |
| `COPY_DST` | CPU → GPU copy | — |
| `MAP_READ` | CPU read from GPU | Slow (PCIe) |
| `MAP_WRITE` | CPU write to GPU | Slow (PCIe) |

### Optimal Pattern

```
CPU → (COPY_DST) → GPU Storage → (compute) → GPU Storage → (COPY_SRC + MAP_READ) → CPU
```

```rust
let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("input"),
    contents: bytemuck::cast_slice(&cpu_data),
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
});

let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("output"),
    size: buffer_size,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
});

// Readback buffer
let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("readback"),
    size: buffer_size,
    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
});
```

### Staging Buffer

For frequent CPU→GPU transfers, use a staging buffer:

```rust
// Write to staging (CPU-visible), then copy to GPU storage
let staging = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("staging"),
    contents: bytemuck::cast_slice(&data),
    usage: wgpu::BufferUsages::COPY_SRC,
});

encoder.copy_buffer_to_buffer(&staging, 0, &storage_buffer, 0, size);
```

## Texture Management

### TexturePool (from this crate)

```rust
use aod_ae_utils::gpu::TexturePool;

let mut pool = TexturePool::new(wgpu::TextureFormat::Rgba8Unorm);

// Reuse textures across frames
let tex = pool.acquire(device, width, height);
// ... use texture ...
pool.release(tex); // Return for reuse
```

### Texture Upload

```rust
queue.write_texture(
    wgpu::TexelCopyTextureInfo {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    },
    pixel_data,
    wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(4 * width),
        rows_per_image: Some(height),
    },
    wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
);
```

## Workgroup Sizing

```rust
// Compute optimal workgroup dimensions
fn optimal_workgroup(width: u32, height: u32, max_size: u32) -> (u32, u32, u32) {
    let max_per_dim = (max_size as f32).sqrt() as u32;
    let x = width.min(max_per_dim).max(1);
    let y = (height / x).min(max_per_dim).max(1);
    (x, y, 1)
}
```

Typical max workgroup size: 256 (mobile) to 1024 (desktop). Query `device.limits().max_compute_workgroup_size_x`.

## Performance Tips

1. **Minimize CPU↔GPU transfers** — batch uploads, process entire frame on GPU
2. **Use texture pools** — avoid allocation overhead
3. **Workgroup size 256** — good default for most GPUs
4. **Avoid read-modify-write** — use separate input/output buffers when possible
5. **Prefill textures** — reuse across frames when content doesn't change
6. **Profile** — use `wgpu`'s `TimestampQuery` or platform-specific tools

## AE Integration

AE runs on the CPU thread. To use GPU:

1. Initialize `GpuContext` once at plugin load (store in global)
2. In render callback: upload → dispatch → readback → write to AE layer
3. Synchronize with `device.poll(wgpu::Maintain::Wait)` if needed

**Gotcha:** wgpu 28 removed `wgpu::Maintain` — use `device.poll(Maintain::Wait)` or the async approach.
