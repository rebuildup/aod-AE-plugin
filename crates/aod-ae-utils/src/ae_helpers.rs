use after_effects as ae;

// ============================================================================
// Layer Accessor
// ============================================================================

pub struct LayerAccessor<'a> {
    layer: &'a ae::Layer,
}

impl<'a> LayerAccessor<'a> {
    pub fn new(layer: &'a ae::Layer) -> Self {
        Self { layer }
    }

    pub fn width(&self) -> usize {
        self.layer.width()
    }

    pub fn height(&self) -> usize {
        self.layer.height()
    }

    pub fn pixel_8(&self, x: usize, y: usize) -> &ae::Pixel8 {
        self.layer.as_pixel8(x, y)
    }

    pub fn pixel_16(&self, x: usize, y: usize) -> &ae::Pixel16 {
        self.layer.as_pixel16(x, y)
    }

    pub fn pixel_float(&self, x: usize, y: usize) -> &ae::PixelF32 {
        self.layer.as_pixel32(x, y)
    }

    pub fn bounds_check(&self, x: usize, y: usize) -> bool {
        x < self.layer.width() && y < self.layer.height()
    }

    pub fn bit_depth(&self) -> i16 {
        self.layer.bit_depth()
    }

    pub fn buffer(&self) -> &[u8] {
        self.layer.buffer()
    }

    pub fn row_bytes(&self) -> isize {
        self.layer.row_bytes()
    }
}

pub struct LayerAccessorMut<'a> {
    layer: &'a mut ae::Layer,
}

impl<'a> LayerAccessorMut<'a> {
    pub fn new(layer: &'a mut ae::Layer) -> Self {
        Self { layer }
    }

    pub fn width(&self) -> usize {
        self.layer.width()
    }

    pub fn height(&self) -> usize {
        self.layer.height()
    }

    pub fn set_pixel_float(&mut self, x: usize, y: usize, pixel: ae::PixelF32) {
        *self.layer.as_pixel32_mut(x, y) = pixel;
    }

    pub fn set_pixel_8(&mut self, x: usize, y: usize, pixel: ae::Pixel8) {
        *self.layer.as_pixel8_mut(x, y) = pixel;
    }

    pub fn set_pixel_16(&mut self, x: usize, y: usize, pixel: ae::Pixel16) {
        *self.layer.as_pixel16_mut(x, y) = pixel;
    }

    pub fn fill(&mut self, color: Option<ae::Pixel8>) -> Result<(), ae::Error> {
        self.layer.fill(color, None)
    }

    pub fn fill16(&mut self, color: Option<ae::Pixel16>) -> Result<(), ae::Error> {
        self.layer.fill16(color, None)
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        self.layer.buffer_mut()
    }
}

// ============================================================================
// Typed Parameter Reader (generic over param enum type)
// ============================================================================

pub fn get_float_param<P: Eq + std::hash::Hash + Copy + std::fmt::Debug>(
    params: &ae::Parameters<P>,
    param_type: P,
) -> Result<f64, ae::Error> {
    Ok(params.get(param_type)?.as_float_slider()?.value())
}

pub fn get_int_param<P: Eq + std::hash::Hash + Copy + std::fmt::Debug>(
    params: &ae::Parameters<P>,
    param_type: P,
) -> Result<i32, ae::Error> {
    Ok(params.get(param_type)?.as_slider()?.value())
}

pub fn get_color_param<P: Eq + std::hash::Hash + Copy + std::fmt::Debug>(
    params: &ae::Parameters<P>,
    param_type: P,
) -> Result<ae::PixelF32, ae::Error> {
    params.get(param_type)?.as_color()?.float_value()
}

pub fn get_popup_param<P: Eq + std::hash::Hash + Copy + std::fmt::Debug>(
    params: &ae::Parameters<P>,
    param_type: P,
) -> Result<i32, ae::Error> {
    Ok(params.get(param_type)?.as_popup()?.value())
}

pub fn get_checkbox_param<P: Eq + std::hash::Hash + Copy + std::fmt::Debug>(
    params: &ae::Parameters<P>,
    param_type: P,
) -> Result<bool, ae::Error> {
    Ok(params.get(param_type)?.as_checkbox()?.value())
}

pub fn get_angle_param<P: Eq + std::hash::Hash + Copy + std::fmt::Debug>(
    params: &ae::Parameters<P>,
    param_type: P,
) -> Result<f64, ae::Error> {
    Ok(params.get(param_type)?.as_angle()?.value().into())
}

pub fn get_point_param<P: Eq + std::hash::Hash + Copy + std::fmt::Debug>(
    params: &ae::Parameters<P>,
    param_type: P,
) -> Result<(f32, f32), ae::Error> {
    Ok(params.get(param_type)?.as_point()?.value())
}

// ============================================================================
// Cache Helpers
// ============================================================================

pub fn create_cache_key(frame: i32, param_hash: u64) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    frame.hash(&mut hasher);
    param_hash.hash(&mut hasher);
    hasher.finish()
}

// ============================================================================
// Error Helpers
// ============================================================================

pub fn log_error(msg: &str) {
    log::error!("[aod-ae-utils] {}", msg);
}

pub fn log_info(msg: &str) {
    log::info!("[aod-ae-utils] {}", msg);
}

pub fn log_debug(msg: &str) {
    log::debug!("[aod-ae-utils] {}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_deterministic() {
        let k1 = create_cache_key(100, 0x1234);
        let k2 = create_cache_key(100, 0x1234);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_different_frames() {
        let k1 = create_cache_key(0, 0x1234);
        let k2 = create_cache_key(1, 0x1234);
        assert_ne!(k1, k2);
    }
}
