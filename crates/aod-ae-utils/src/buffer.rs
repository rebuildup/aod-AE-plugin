use after_effects as ae;

// ============================================================================
// Region of Interest
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct RegionOfInterest {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl RegionOfInterest {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn full(width: usize, height: usize) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn area(&self) -> usize {
        self.width * self.height
    }
}

// ============================================================================
// Image Buffer
// ============================================================================

pub struct ImageBuffer<P: Copy> {
    data: Vec<P>,
    width: usize,
    height: usize,
}

impl<P: Copy> ImageBuffer<P> {
    pub fn from_raw(data: Vec<P>, width: usize, height: usize) -> Option<Self> {
        if data.len() == width * height {
            Some(Self {
                data,
                width,
                height,
            })
        } else {
            None
        }
    }

    pub fn from_fn(width: usize, height: usize, f: impl Fn(usize, usize) -> P) -> Self {
        let mut data = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                data.push(f(x, y));
            }
        }
        Self {
            data,
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn pixels(&self) -> &[P] {
        &self.data
    }
    pub fn pixels_mut(&mut self) -> &mut [P] {
        &mut self.data
    }

    pub fn pixel(&self, x: usize, y: usize) -> Option<&P> {
        if x < self.width && y < self.height {
            Some(&self.data[y * self.width + x])
        } else {
            None
        }
    }

    pub fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut P> {
        if x < self.width && y < self.height {
            Some(&mut self.data[y * self.width + x])
        } else {
            None
        }
    }

    pub fn row(&self, y: usize) -> Option<&[P]> {
        if y < self.height {
            let start = y * self.width;
            Some(&self.data[start..start + self.width])
        } else {
            None
        }
    }

    pub fn row_mut(&mut self, y: usize) -> Option<&mut [P]> {
        if y < self.height {
            let start = y * self.width;
            Some(&mut self.data[start..start + self.width])
        } else {
            None
        }
    }

    pub fn region(&self, roi: &RegionOfInterest) -> Option<ImageBufferView<'_, P>> {
        if roi.x + roi.width <= self.width && roi.y + roi.height <= self.height {
            Some(ImageBufferView {
                buffer: self,
                roi: *roi,
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &P> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut P> {
        self.data.iter_mut()
    }

    pub fn iter_enumerated(&self) -> impl Iterator<Item = (usize, usize, &P)> {
        self.data
            .iter()
            .enumerate()
            .map(move |(i, p)| (i % self.width, i / self.width, p))
    }

    pub fn map_in_place(&mut self, f: impl Fn(P) -> P) {
        for pixel in self.data.iter_mut() {
            *pixel = f(*pixel);
        }
    }

    pub fn fill(&mut self, value: P) {
        for pixel in self.data.iter_mut() {
            *pixel = value;
        }
    }
}

// ============================================================================
// Image Buffer View (zero-copy)
// ============================================================================

pub struct ImageBufferView<'a, P: Copy> {
    buffer: &'a ImageBuffer<P>,
    roi: RegionOfInterest,
}

impl<'a, P: Copy> ImageBufferView<'a, P> {
    pub fn width(&self) -> usize {
        self.roi.width
    }
    pub fn height(&self) -> usize {
        self.roi.height
    }

    pub fn pixel(&self, x: usize, y: usize) -> Option<&P> {
        if x < self.roi.width && y < self.roi.height {
            self.buffer.pixel(x + self.roi.x, y + self.roi.y)
        } else {
            None
        }
    }

    pub fn row(&self, y: usize) -> Option<&[P]> {
        if y < self.roi.height {
            let full_row = self.buffer.row(y + self.roi.y)?;
            Some(&full_row[self.roi.x..self.roi.x + self.roi.width])
        } else {
            None
        }
    }
}

// ============================================================================
// Pixel Type Aliases
// ============================================================================

pub type Image8 = ImageBuffer<ae::Pixel8>;
pub type Image16 = ImageBuffer<ae::Pixel16>;
pub type ImageF32 = ImageBuffer<ae::PixelF32>;

// ============================================================================
// AE Layer Conversion
// ============================================================================

impl ImageF32 {
    /// Create from AE Layer (reads pixel data via buffer access)
    pub fn from_layer(layer: &ae::Layer) -> Option<Self> {
        let width = layer.width();
        let height = layer.height();
        let buffer = Self::from_fn(width, height, |x, y| *layer.as_pixel32(x, y));
        Some(buffer)
    }

    /// Write back to AE Layer (float)
    pub fn write_to_layer(&self, layer: &mut ae::Layer) {
        for y in 0..self.height {
            if let Some(src_row) = self.row(y) {
                for (x, pixel) in src_row.iter().enumerate() {
                    *layer.as_pixel32_mut(x, y) = *pixel;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_buffer_creation() {
        let buf = ImageF32::from_fn(100, 100, |x, y| ae::PixelF32 {
            red: x as f32 / 100.0,
            green: y as f32 / 100.0,
            blue: 0.0,
            alpha: 1.0,
        });
        assert_eq!(buf.width(), 100);
        assert_eq!(buf.height(), 100);
        assert_eq!(buf.pixels().len(), 10000);
    }

    #[test]
    fn test_region_of_interest() {
        let roi = RegionOfInterest::new(10, 20, 50, 60);
        assert!(roi.contains(10, 20));
        assert!(roi.contains(59, 79));
        assert!(!roi.contains(9, 20));
        assert!(!roi.contains(10, 80));
        assert_eq!(roi.area(), 3000);
    }
}
