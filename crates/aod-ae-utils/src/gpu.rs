// ============================================================================
// GPU Helpers (wgpu-based)
//
// This module provides abstractions for GPU-accelerated image processing
// in After Effects plugins using wgpu.
// ============================================================================

#[cfg(feature = "gpu")]
use wgpu::util::DeviceExt;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum GpuError {
    DeviceNotFound,
    AdapterRequestFailed,
    SurfaceError(String),
    PipelineError(String),
    BufferError(String),
    TextureError(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::DeviceNotFound => write!(f, "No suitable GPU device found"),
            GpuError::AdapterRequestFailed => write!(f, "Failed to request GPU adapter"),
            GpuError::SurfaceError(e) => write!(f, "Surface error: {}", e),
            GpuError::PipelineError(e) => write!(f, "Pipeline error: {}", e),
            GpuError::BufferError(e) => write!(f, "Buffer error: {}", e),
            GpuError::TextureError(e) => write!(f, "Texture error: {}", e),
        }
    }
}

impl std::error::Error for GpuError {}

// ============================================================================
// GpuContext
// ============================================================================

#[cfg(feature = "gpu")]
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

#[cfg(feature = "gpu")]
impl GpuContext {
    pub async fn new() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| GpuError::AdapterRequestFailed)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("aod-ae-utils GPU"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::PipelineError(e.to_string()))?;

        Ok(Self { device, queue })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

// ============================================================================
// Texture Pool
// ============================================================================

#[cfg(feature = "gpu")]
pub struct TexturePool {
    textures: Vec<wgpu::Texture>,
    format: wgpu::TextureFormat,
}

#[cfg(feature = "gpu")]
impl TexturePool {
    pub fn new(format: wgpu::TextureFormat) -> Self {
        Self {
            textures: Vec::new(),
            format,
        }
    }

    pub fn acquire(&mut self, device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        if let Some(tex) = self.textures.pop() {
            if tex.width() >= width && tex.height() >= height {
                return tex;
            }
        }

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("aod-ae-utils texture pool"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    pub fn release(&mut self, texture: wgpu::Texture) {
        self.textures.push(texture);
    }

    pub fn clear(&mut self) {
        self.textures.clear();
    }

    pub fn pool_size(&self) -> usize {
        self.textures.len()
    }
}

// ============================================================================
// Buffer Helpers
// ============================================================================

#[cfg(feature = "gpu")]
pub fn create_buffer_from_data(
    device: &wgpu::Device,
    data: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("aod-ae-utils buffer"),
        contents: data,
        usage,
    })
}

// ============================================================================
// Compute Pipeline Cache
// ============================================================================

#[cfg(feature = "gpu")]
pub struct ComputePipelineCache {
    pipelines: std::collections::HashMap<String, wgpu::ComputePipeline>,
}

#[cfg(feature = "gpu")]
impl Default for ComputePipelineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputePipelineCache {
    pub fn new() -> Self {
        Self {
            pipelines: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        key: &str,
        shader_source: &str,
        entry_point: &str,
    ) -> Result<&wgpu::ComputePipeline, GpuError> {
        if !self.pipelines.contains_key(key) {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(key),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(key),
                layout: None,
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: None,
            });

            self.pipelines.insert(key.to_string(), pipeline);
        }

        Ok(self.pipelines.get(key).unwrap())
    }

    pub fn remove(&mut self, key: &str) {
        self.pipelines.remove(key);
    }

    pub fn clear(&mut self) {
        self.pipelines.clear();
    }
}

// ============================================================================
// Image Processing Helpers
// ============================================================================

/// Upload pixel data to a GPU texture
#[cfg(feature = "gpu")]
pub fn upload_pixels_to_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pixel upload texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    texture
}

// ============================================================================
// Workgroup Utilities
// ============================================================================

/// Calculate optimal workgroup size for a given image dimension
pub fn optimal_workgroup_size(width: u32, height: u32, max_workgroup_size: u32) -> (u32, u32, u32) {
    let max_per_dim = (max_workgroup_size as f32).sqrt() as u32;
    let x = width.min(max_per_dim).max(1);
    let y = (height / x).min(max_per_dim).max(1);
    (x, y, 1)
}

/// Round up to next power of 2
pub fn next_power_of_2(v: u32) -> u32 {
    v.next_power_of_two()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_workgroup_size() {
        let (x, y, z) = optimal_workgroup_size(1920, 1080, 256);
        assert!(x * y * z <= 256);
        assert!(x > 0);
        assert!(y > 0);
    }

    #[test]
    fn test_next_power_of_2() {
        assert_eq!(next_power_of_2(1), 1);
        assert_eq!(next_power_of_2(3), 4);
        assert_eq!(next_power_of_2(8), 8);
        assert_eq!(next_power_of_2(100), 128);
    }
}
