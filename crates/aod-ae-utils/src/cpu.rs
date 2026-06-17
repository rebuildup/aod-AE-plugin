use after_effects as ae;

// ============================================================================
// Parallel Processor
// ============================================================================

#[cfg(feature = "cpu")]
use rayon::prelude::*;

pub struct ParallelProcessor {
    chunk_size: usize,
}

impl ParallelProcessor {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    pub fn default_processor() -> Self {
        Self::new(4096)
    }

    #[cfg(feature = "cpu")]
    pub fn process_parallel<F>(&self, pixels: &mut [ae::PixelF32], f: F)
    where
        F: Fn(&mut ae::PixelF32) + Sync + Send,
    {
        pixels.par_chunks_mut(self.chunk_size).for_each(|chunk| {
            for pixel in chunk.iter_mut() {
                f(pixel);
            }
        });
    }

    #[cfg(not(feature = "cpu"))]
    pub fn process_parallel<F>(&self, pixels: &mut [ae::PixelF32], f: F)
    where
        F: Fn(&mut ae::PixelF32) + Sync + Send,
    {
        for pixel in pixels.iter_mut() {
            f(pixel);
        }
    }

    #[cfg(feature = "cpu")]
    pub fn process_parallel_with_coords<F>(&self, pixels: &mut [ae::PixelF32], width: usize, f: F)
    where
        F: Fn(usize, usize, &mut ae::PixelF32) + Sync + Send,
    {
        pixels
            .par_chunks_mut(self.chunk_size)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                let start_idx = chunk_idx * self.chunk_size;
                for (i, pixel) in chunk.iter_mut().enumerate() {
                    let idx = start_idx + i;
                    let x = idx % width;
                    let y = idx / width;
                    f(x, y, pixel);
                }
            });
    }

    #[cfg(not(feature = "cpu"))]
    pub fn process_parallel_with_coords<F>(&self, pixels: &mut [ae::PixelF32], width: usize, f: F)
    where
        F: Fn(usize, usize, &mut ae::PixelF32) + Sync + Send,
    {
        for (i, pixel) in pixels.iter_mut().enumerate() {
            let x = i % width;
            let y = i / width;
            f(x, y, pixel);
        }
    }
}

// ============================================================================
// Chunk Iterator (cache-friendly)
// ============================================================================

pub struct ChunkIterator<'a, T> {
    slice: &'a [T],
    chunk_size: usize,
    pos: usize,
}

impl<'a, T> ChunkIterator<'a, T> {
    pub fn new(slice: &'a [T], chunk_size: usize) -> Self {
        Self {
            slice,
            chunk_size,
            pos: 0,
        }
    }
}

impl<'a, T> Iterator for ChunkIterator<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.slice.len() {
            return None;
        }
        let end = (self.pos + self.chunk_size).min(self.slice.len());
        let chunk = &self.slice[self.pos..end];
        self.pos = end;
        Some(chunk)
    }
}

// ============================================================================
// Tile Processor
// ============================================================================

pub struct TileProcessor {
    tile_size: usize,
}

impl TileProcessor {
    pub fn new(tile_size: usize) -> Self {
        Self { tile_size }
    }

    pub fn default_processor() -> Self {
        Self::new(64)
    }

    pub fn for_each_tile(
        &self,
        width: usize,
        height: usize,
        mut f: impl FnMut(usize, usize, usize, usize),
    ) {
        for ty in (0..height).step_by(self.tile_size) {
            for tx in (0..width).step_by(self.tile_size) {
                let tw = (self.tile_size).min(width - tx);
                let th = (self.tile_size).min(height - ty);
                f(tx, ty, tw, th);
            }
        }
    }

    #[cfg(feature = "cpu")]
    pub fn for_each_tile_parallel(
        &self,
        width: usize,
        height: usize,
        f: impl Fn(usize, usize, usize, usize) + Sync + Send,
    ) {
        let tiles: Vec<(usize, usize)> = (0..height)
            .step_by(self.tile_size)
            .flat_map(|ty| (0..width).step_by(self.tile_size).map(move |tx| (tx, ty)))
            .collect();

        tiles.par_iter().for_each(|&(tx, ty)| {
            let tw = self.tile_size.min(width - tx);
            let th = self.tile_size.min(height - ty);
            f(tx, ty, tw, th);
        });
    }
}

// ============================================================================
// SIMD Helpers (when available)
// ============================================================================

#[cfg(target_arch = "x86_64")]
pub mod simd {
    #[cfg(target_feature = "avx2")]
    use std::arch::x86_64::*;

    /// Add two pixel arrays using SIMD (f32x8)
    #[cfg(target_feature = "avx2")]
    pub unsafe fn add_pixels_avx2(a: &mut [f32], b: &[f32]) {
        let chunks = a.len() / 8;
        for i in 0..chunks {
            let va = _mm256_loadu_ps(a.as_ptr().add(i * 8));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i * 8));
            let vc = _mm256_add_ps(va, vb);
            _mm256_storeu_ps(a.as_mut_ptr().add(i * 8), vc);
        }
    }

    /// Multiply pixel array by scalar using SIMD
    #[cfg(target_feature = "avx2")]
    pub unsafe fn scale_pixels_avx2(pixels: &mut [f32], scale: f32) {
        let v_scale = _mm256_set1_ps(scale);
        let chunks = pixels.len() / 8;
        for i in 0..chunks {
            let va = _mm256_loadu_ps(pixels.as_ptr().add(i * 8));
            let vc = _mm256_mul_ps(va, v_scale);
            _mm256_storeu_ps(pixels.as_mut_ptr().add(i * 8), vc);
        }
    }

    /// Clamp pixel array to [0, 1] using SIMD
    #[cfg(target_feature = "avx2")]
    pub unsafe fn clamp_pixels_avx2(pixels: &mut [f32]) {
        let zero = _mm256_setzero_ps();
        let one = _mm256_set1_ps(1.0);
        let chunks = pixels.len() / 8;
        for i in 0..chunks {
            let va = _mm256_loadu_ps(pixels.as_ptr().add(i * 8));
            let vc = _mm256_max_ps(_mm256_min_ps(va, one), zero);
            _mm256_storeu_ps(pixels.as_mut_ptr().add(i * 8), vc);
        }
    }
}

// ============================================================================
// Memory Utilities
// ============================================================================

/// Alignment helper for SIMD
pub fn align_to(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

/// Check if pointer is aligned
pub fn is_aligned<T>(ptr: *const T, alignment: usize) -> bool {
    (ptr as usize).is_multiple_of(alignment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_processor() {
        let mut pixels: Vec<ae::PixelF32> = (0..100)
            .map(|i| ae::PixelF32 {
                red: i as f32 / 100.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            })
            .collect();

        let processor = ParallelProcessor::new(16);
        processor.process_parallel(&mut pixels, |p| {
            p.green = p.red * 2.0;
        });

        for (i, p) in pixels.iter().enumerate() {
            assert!((p.green - (i as f32 / 100.0 * 2.0)).abs() < 0.001);
        }
    }

    #[test]
    fn test_chunk_iterator() {
        let data: Vec<i32> = (0..10).collect();
        let chunks: Vec<&[i32]> = ChunkIterator::new(&data, 3).collect();
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], &[0, 1, 2]);
        assert_eq!(chunks[3], &[9]);
    }

    #[test]
    fn test_tile_processor() {
        let mut tiles = Vec::new();
        let processor = TileProcessor::new(16);
        processor.for_each_tile(100, 100, |x, y, w, h| {
            tiles.push((x, y, w, h));
        });
        assert!(!tiles.is_empty());
        assert_eq!(tiles[0], (0, 0, 16, 16));
    }

    #[test]
    fn test_align_to() {
        assert_eq!(align_to(0, 32), 0);
        assert_eq!(align_to(1, 32), 32);
        assert_eq!(align_to(31, 32), 32);
        assert_eq!(align_to(32, 32), 32);
        assert_eq!(align_to(33, 32), 64);
    }
}
