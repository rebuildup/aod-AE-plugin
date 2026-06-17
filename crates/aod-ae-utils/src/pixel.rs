use after_effects as ae;
use std::marker::PhantomData;

// ============================================================================
// Pixel Conversion (existing)
// ============================================================================

pub trait ToPixel {
    fn to_pixel32(&self) -> ae::PixelF32;
    fn to_pixel16(&self) -> ae::Pixel16;
    fn to_pixel8(&self) -> ae::Pixel8;
}

impl ToPixel for ae::sys::PF_Pixel {
    fn to_pixel32(&self) -> ae::PixelF32 {
        ae::PixelF32 {
            red: self.red as f32 / ae::MAX_CHANNEL8 as f32,
            green: self.green as f32 / ae::MAX_CHANNEL8 as f32,
            blue: self.blue as f32 / ae::MAX_CHANNEL8 as f32,
            alpha: self.alpha as f32 / ae::MAX_CHANNEL8 as f32,
        }
    }

    fn to_pixel16(&self) -> ae::Pixel16 {
        ae::Pixel16 {
            red: (self.red as f32 / ae::MAX_CHANNEL8 as f32 * ae::MAX_CHANNEL16 as f32) as u16,
            green: (self.green as f32 / ae::MAX_CHANNEL8 as f32 * ae::MAX_CHANNEL16 as f32) as u16,
            blue: (self.blue as f32 / ae::MAX_CHANNEL8 as f32 * ae::MAX_CHANNEL16 as f32) as u16,
            alpha: (self.alpha as f32 / ae::MAX_CHANNEL8 as f32 * ae::MAX_CHANNEL16 as f32) as u16,
        }
    }

    fn to_pixel8(&self) -> ae::Pixel8 {
        ae::Pixel8 {
            red: self.red,
            green: self.green,
            blue: self.blue,
            alpha: self.alpha,
        }
    }
}

impl ToPixel for ae::Pixel16 {
    fn to_pixel32(&self) -> ae::PixelF32 {
        ae::PixelF32 {
            red: self.red as f32 / ae::MAX_CHANNEL16 as f32,
            green: self.green as f32 / ae::MAX_CHANNEL16 as f32,
            blue: self.blue as f32 / ae::MAX_CHANNEL16 as f32,
            alpha: self.alpha as f32 / ae::MAX_CHANNEL16 as f32,
        }
    }
    fn to_pixel16(&self) -> ae::Pixel16 {
        *self
    }
    fn to_pixel8(&self) -> ae::Pixel8 {
        ae::Pixel8 {
            red: (self.red as f32 / ae::MAX_CHANNEL16 as f32 * ae::MAX_CHANNEL8 as f32) as u8,
            green: (self.green as f32 / ae::MAX_CHANNEL16 as f32 * ae::MAX_CHANNEL8 as f32) as u8,
            blue: (self.blue as f32 / ae::MAX_CHANNEL16 as f32 * ae::MAX_CHANNEL8 as f32) as u8,
            alpha: (self.alpha as f32 / ae::MAX_CHANNEL16 as f32 * ae::MAX_CHANNEL8 as f32) as u8,
        }
    }
}

impl ToPixel for ae::PixelF32 {
    fn to_pixel32(&self) -> ae::PixelF32 {
        *self
    }
    fn to_pixel16(&self) -> ae::Pixel16 {
        ae::Pixel16 {
            red: (self.red.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
            green: (self.green.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
            blue: (self.blue.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
            alpha: (self.alpha.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
        }
    }
    fn to_pixel8(&self) -> ae::Pixel8 {
        ae::Pixel8 {
            red: (self.red.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
            green: (self.green.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
            blue: (self.blue.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
            alpha: (self.alpha.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
        }
    }
}

// ============================================================================
// Pixel Blending
// ============================================================================

pub trait PixelBlender {
    fn normal(self, other: Self) -> Self;
    fn multiply(self, other: Self) -> Self;
    fn screen(self, other: Self) -> Self;
    fn overlay(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;
    fn subtract(self, other: Self) -> Self;
    fn difference(self, other: Self) -> Self;
    fn darken(self, other: Self) -> Self;
    fn lighten(self, other: Self) -> Self;
}

fn blend_channel(a: f32, b: f32, f: impl Fn(f32, f32) -> f32) -> f32 {
    f(a, b).clamp(0.0, 1.0)
}

impl PixelBlender for ae::PixelF32 {
    fn normal(self, other: Self) -> Self {
        let alpha = other.alpha;
        let inv = 1.0 - alpha;
        Self {
            red: self.red * inv + other.red * alpha,
            green: self.green * inv + other.green * alpha,
            blue: self.blue * inv + other.blue * alpha,
            alpha: (self.alpha + other.alpha).clamp(0.0, 1.0),
        }
    }

    fn multiply(self, other: Self) -> Self {
        Self {
            red: self.red * other.red,
            green: self.green * other.green,
            blue: self.blue * other.blue,
            alpha: self.alpha * other.alpha,
        }
    }

    fn screen(self, other: Self) -> Self {
        Self {
            red: 1.0 - (1.0 - self.red) * (1.0 - other.red),
            green: 1.0 - (1.0 - self.green) * (1.0 - other.green),
            blue: 1.0 - (1.0 - self.blue) * (1.0 - other.blue),
            alpha: 1.0 - (1.0 - self.alpha) * (1.0 - other.alpha),
        }
    }

    fn overlay(self, other: Self) -> Self {
        Self {
            red: blend_channel(self.red, other.red, |a, b| {
                if a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            }),
            green: blend_channel(self.green, other.green, |a, b| {
                if a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            }),
            blue: blend_channel(self.blue, other.blue, |a, b| {
                if a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            }),
            alpha: self.alpha,
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            red: (self.red + other.red).clamp(0.0, 1.0),
            green: (self.green + other.green).clamp(0.0, 1.0),
            blue: (self.blue + other.blue).clamp(0.0, 1.0),
            alpha: (self.alpha + other.alpha).clamp(0.0, 1.0),
        }
    }

    fn subtract(self, other: Self) -> Self {
        Self {
            red: (self.red - other.red).clamp(0.0, 1.0),
            green: (self.green - other.green).clamp(0.0, 1.0),
            blue: (self.blue - other.blue).clamp(0.0, 1.0),
            alpha: self.alpha,
        }
    }

    fn difference(self, other: Self) -> Self {
        Self {
            red: (self.red - other.red).abs(),
            green: (self.green - other.green).abs(),
            blue: (self.blue - other.blue).abs(),
            alpha: self.alpha,
        }
    }

    fn darken(self, other: Self) -> Self {
        Self {
            red: self.red.min(other.red),
            green: self.green.min(other.green),
            blue: self.blue.min(other.blue),
            alpha: self.alpha,
        }
    }

    fn lighten(self, other: Self) -> Self {
        Self {
            red: self.red.max(other.red),
            green: self.green.max(other.green),
            blue: self.blue.max(other.blue),
            alpha: self.alpha,
        }
    }
}

// ============================================================================
// Pixel Map Iterator
// ============================================================================

pub struct PixelMap<'a, I, F> {
    iter: I,
    f: F,
    _marker: PhantomData<&'a ()>,
}

impl<'a, I, F> PixelMap<'a, I, F> {
    pub fn new(iter: I, f: F) -> Self {
        Self {
            iter,
            f,
            _marker: PhantomData,
        }
    }
}

impl<'a, I, F> Iterator for PixelMap<'a, I, F>
where
    I: Iterator<Item = &'a mut ae::PixelF32>,
    F: FnMut(&mut ae::PixelF32),
{
    type Item = &'a mut ae::PixelF32;

    fn next(&mut self) -> Option<Self::Item> {
        let pixel = self.iter.next()?;
        (self.f)(pixel);
        Some(pixel)
    }
}

// ============================================================================
// Channel Operations
// ============================================================================

pub fn split_rgba(pixel: &ae::PixelF32) -> [f32; 4] {
    [pixel.red, pixel.green, pixel.blue, pixel.alpha]
}

pub fn combine_rgba(channels: [f32; 4]) -> ae::PixelF32 {
    ae::PixelF32 {
        red: channels[0],
        green: channels[1],
        blue: channels[2],
        alpha: channels[3],
    }
}

pub fn extract_red(buffer: &mut [ae::PixelF32]) -> Vec<f32> {
    buffer.iter().map(|p| p.red).collect()
}

pub fn extract_green(buffer: &mut [ae::PixelF32]) -> Vec<f32> {
    buffer.iter().map(|p| p.green).collect()
}

pub fn extract_blue(buffer: &mut [ae::PixelF32]) -> Vec<f32> {
    buffer.iter().map(|p| p.blue).collect()
}

pub fn extract_alpha(buffer: &mut [ae::PixelF32]) -> Vec<f32> {
    buffer.iter().map(|p| p.alpha).collect()
}

pub fn apply_to_channel(buffer: &mut [ae::PixelF32], channel: usize, f: impl Fn(f32) -> f32) {
    for pixel in buffer.iter_mut() {
        match channel {
            0 => pixel.red = f(pixel.red),
            1 => pixel.green = f(pixel.green),
            2 => pixel.blue = f(pixel.blue),
            3 => pixel.alpha = f(pixel.alpha),
            _ => {}
        }
    }
}

pub fn luminance(pixel: &ae::PixelF32) -> f32 {
    0.2126 * pixel.red + 0.7152 * pixel.green + 0.0722 * pixel.blue
}

pub fn grayscale(pixel: &ae::PixelF32) -> ae::PixelF32 {
    let l = luminance(pixel);
    ae::PixelF32 {
        red: l,
        green: l,
        blue: l,
        alpha: pixel.alpha,
    }
}

pub fn clamp_pixel(pixel: &mut ae::PixelF32) {
    pixel.red = pixel.red.clamp(0.0, 1.0);
    pixel.green = pixel.green.clamp(0.0, 1.0);
    pixel.blue = pixel.blue.clamp(0.0, 1.0);
    pixel.alpha = pixel.alpha.clamp(0.0, 1.0);
}

pub fn premultiply_alpha(pixel: &mut ae::PixelF32) {
    pixel.red *= pixel.alpha;
    pixel.green *= pixel.alpha;
    pixel.blue *= pixel.alpha;
}

pub fn unpremultiply_alpha(pixel: &mut ae::PixelF32) {
    if pixel.alpha > 0.0 {
        pixel.red /= pixel.alpha;
        pixel.green /= pixel.alpha;
        pixel.blue /= pixel.alpha;
    }
}
