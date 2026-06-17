// ============================================================================
// Color Space Conversions
// ============================================================================

/// Convert sRGB gamma-encoded value to linear
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear value to sRGB gamma-encoded
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Apply gamma correction
pub fn gamma_correct(c: f32, gamma: f32) -> f32 {
    c.clamp(0.0, 1.0).powf(1.0 / gamma)
}

// ============================================================================
// RGB <-> HSV
// ============================================================================

/// Convert RGB to HSV (H: 0-360, S: 0-1, V: 0-1)
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { d / max };

    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, v)
}

/// Convert HSV to RGB (H: 0-360, S: 0-1, V: 0-1)
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}

// ============================================================================
// RGB <-> HSL
// ============================================================================

/// Convert RGB to HSL (H: 0-360, S: 0-1, L: 0-1)
pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, l)
}

/// Convert HSL to RGB (H: 0-360, S: 0-1, L: 0-1)
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let hue_to_rgb = |p: f32, q: f32, t: f32| -> f32 {
        let t = if t < 0.0 {
            t + 360.0
        } else if t > 360.0 {
            t - 360.0
        } else {
            t
        };
        if t < 60.0 {
            p + (q - p) * t / 60.0
        } else if t < 180.0 {
            q
        } else if t < 240.0 {
            p + (q - p) * (240.0 - t) / 60.0
        } else {
            p
        }
    };

    let r = hue_to_rgb(p, q, h + 120.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 120.0);

    (r, g, b)
}

// ============================================================================
// RGB <-> OKLCH (Perceptual Color Space)
// ============================================================================

/// Linear sRGB to OKLab
pub fn linear_srgb_to_oklab(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let l = 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_;
    let a = 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_;
    let b = 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_;

    (l, a, b)
}

/// OKLab to linear sRGB
pub fn oklab_to_linear_srgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

    let l_ = l_ * l_ * l_;
    let m_ = m_ * m_ * m_;
    let s_ = s_ * s_ * s_;

    let r = 4.076_741_7 * l_ - 3.307_711_6 * m_ + 0.230_969_94 * s_;
    let g = -1.268_438 * l_ + 2.609_757_4 * m_ - 0.341_319_38 * s_;
    let bl = -0.0041960863 * l_ - 0.703_418_6 * m_ + 1.707_614_7 * s_;

    (r, g, bl)
}

/// Convert RGB to OKLCH (L: 0-1, C: 0-~0.4, H: 0-360)
pub fn rgb_to_oklch(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r_lin = srgb_to_linear(r);
    let g_lin = srgb_to_linear(g);
    let b_lin = srgb_to_linear(b);
    let (l, a, bl) = linear_srgb_to_oklab(r_lin, g_lin, b_lin);
    let c = (a * a + bl * bl).sqrt();
    let h = bl.atan2(a).to_degrees();
    let h = if h < 0.0 { h + 360.0 } else { h };
    (l, c, h)
}

/// Convert OKLCH to RGB (L: 0-1, C: 0-~0.4, H: 0-360)
/// Note: Colors outside sRGB gamut will be clamped to [0, 1].
pub fn oklch_to_rgb(l: f32, c: f32, h: f32) -> (f32, f32, f32) {
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    let (r, g, bl) = oklab_to_linear_srgb(l, a, b);
    (linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(bl))
}

// ============================================================================
// Luminance
// ============================================================================

/// Rec.709 luminance
pub fn luminance_709(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Rec.2020 luminance
pub fn luminance_2020(r: f32, g: f32, b: f32) -> f32 {
    0.2627 * r + 0.6780 * g + 0.0593 * b
}

/// Perceived lightness from luminance (CIE Lightness)
pub fn perceived_lightness(luminance: f32) -> f32 {
    if luminance <= 0.008856 {
        luminance * 903.3
    } else {
        luminance.powf(1.0 / 3.0) * 116.0 - 16.0
    }
}

// ============================================================================
// Color Temperature
// ============================================================================

/// Convert color temperature (Kelvin) to RGB
pub fn kelvin_to_rgb(kelvin: f32) -> (f32, f32, f32) {
    let temp = kelvin / 100.0;
    let r = if temp <= 66.0 {
        1.0
    } else {
        (temp - 60.0).clamp(0.0, 255.0) / 255.0
    };

    let g = if temp <= 66.0 {
        (99.470_8 * temp.ln() - 161.119_57).clamp(0.0, 255.0) / 255.0
    } else {
        (288.122_16 * (temp - 60.0).ln() - 558.498_4).clamp(0.0, 255.0) / 255.0
    };

    let b = if temp >= 66.0 {
        1.0
    } else if temp <= 19.0 {
        0.0
    } else {
        (138.517_73 * (temp - 10.0).ln() - 305.044_8).clamp(0.0, 255.0) / 255.0
    };

    (r, g, b)
}

// ============================================================================
// Color Utilities
// ============================================================================

/// Mix two colors by factor t (0.0 = a, 1.0 = b)
pub fn mix_color(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
    )
}

/// Invert color (complementary)
pub fn invert_color(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    (1.0 - r, 1.0 - g, 1.0 - b)
}

/// Adjust saturation (0.0 = grayscale, 1.0 = original)
pub fn adjust_saturation(r: f32, g: f32, b: f32, saturation: f32) -> (f32, f32, f32) {
    let l = luminance_709(r, g, b);
    (
        l + (r - l) * saturation,
        l + (g - l) * saturation,
        l + (b - l) * saturation,
    )
}

/// Convert to sepia tone
pub fn sepia(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let gray = luminance_709(r, g, b);
    (
        (gray + 0.1).clamp(0.0, 1.0),
        (gray + 0.0).clamp(0.0, 1.0),
        (gray - 0.1).clamp(0.0, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_linear_roundtrip() {
        for i in 0..=255 {
            let c = i as f32 / 255.0;
            let linear = srgb_to_linear(c);
            let back = linear_to_srgb(linear);
            assert!((c - back).abs() < 0.001);
        }
    }

    #[test]
    fn test_hsv_rgb_roundtrip() {
        let test_cases = [(0.0, 1.0, 1.0), (120.0, 0.5, 0.8), (240.0, 1.0, 0.5)];
        for (h, s, v) in test_cases {
            let (r, g, b) = hsv_to_rgb(h, s, v);
            let (h2, s2, v2) = rgb_to_hsv(r, g, b);
            assert!((h - h2).abs() < 0.01, "H: {} vs {}", h, h2);
            assert!((s - s2).abs() < 0.01, "S: {} vs {}", s, s2);
            assert!((v - v2).abs() < 0.01, "V: {} vs {}", v, v2);
        }
    }

    #[test]
    fn test_hsl_rgb_roundtrip() {
        let test_cases = [(0.0, 1.0, 0.5), (120.0, 0.5, 0.5), (240.0, 0.8, 0.3)];
        for (h, s, l) in test_cases {
            let (r, g, b) = hsl_to_rgb(h, s, l);
            let (h2, s2, l2) = rgb_to_hsl(r, g, b);
            assert!((h - h2).abs() < 0.01, "H: {} vs {}", h, h2);
            assert!((s - s2).abs() < 0.01, "S: {} vs {}", s, s2);
            assert!((l - l2).abs() < 0.01, "L: {} vs {}", l, l2);
        }
    }

    #[test]
    fn test_oklch_roundtrip() {
        // Use values that stay within sRGB gamut
        let test_cases = [(0.6, 0.05, 30.0), (0.7, 0.02, 120.0), (0.5, 0.08, 260.0)];
        for (l, c, h) in test_cases {
            let (r, g, b) = oklch_to_rgb(l, c, h);
            let (l2, c2, h2) = rgb_to_oklch(r, g, b);
            assert!((l - l2).abs() < 0.01, "L: {} vs {}", l, l2);
            assert!((c - c2).abs() < 0.01, "C: {} vs {}", c, c2);
            assert!((h - h2).abs() < 1.0, "H: {} vs {}", h, h2);
        }
    }

    #[test]
    fn test_luminance() {
        let (r, g, b) = (1.0, 1.0, 1.0);
        let l = luminance_709(r, g, b);
        assert!((l - 1.0).abs() < 0.01);
    }
}
