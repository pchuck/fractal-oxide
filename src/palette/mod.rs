use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// Global singleton instances for each palette type
// Using OnceLock ensures thread-safe lazy initialization
static CLASSIC_PALETTE: OnceLock<ClassicPalette> = OnceLock::new();
static FIRE_PALETTE: OnceLock<FirePalette> = OnceLock::new();
static ICE_PALETTE: OnceLock<IcePalette> = OnceLock::new();
static GRAYSCALE_PALETTE: OnceLock<GrayscalePalette> = OnceLock::new();
static PSYCHEDELIC_PALETTE: OnceLock<PsychedelicPalette> = OnceLock::new();

/// Available color palette types for fractal rendering.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum PaletteType {
    #[default]
    Classic,
    Fire,
    Ice,
    Grayscale,
    Psychedelic,
}

/// Trait for color palettes.
///
/// Palettes map a normalized value t (0.0 to 1.0) to a color.
/// For fractals, t typically represents the iteration count ratio.
pub trait Palette: Send + Sync {
    /// Returns the display name of this palette.
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Returns the color for a given normalized value t (0.0 to 1.0).
    fn color(&self, t: f32) -> Color32;
}

/// Classic rainbow gradient palette.
///
/// Colors: black → dark blue → blue → cyan → green → yellow → red → white
/// Provides good contrast and visually pleasing results for most fractals.
pub struct ClassicPalette;

impl Palette for ClassicPalette {
    fn name(&self) -> &str {
        "Classic"
    }

    fn color(&self, t: f32) -> Color32 {
        let colors: [(f32, f32, f32); 8] = [
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 0.5),
            (0.0, 0.0, 1.0),
            (0.0, 1.0, 1.0),
            (0.0, 1.0, 0.0),
            (1.0, 1.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 1.0, 1.0),
        ];

        interpolate_colors(&colors, t)
    }
}

/// Fire-inspired palette.
///
/// Colors: black → dark red → red → orange → yellow → white
/// Creates a heat-map effect, good for highlighting iteration density.
pub struct FirePalette;

impl Palette for FirePalette {
    fn name(&self) -> &str {
        "Fire"
    }

    fn color(&self, t: f32) -> Color32 {
        let colors: [(f32, f32, f32); 6] = [
            (0.0, 0.0, 0.0),
            (0.5, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 0.5, 0.0),
            (1.0, 1.0, 0.0),
            (1.0, 1.0, 1.0),
        ];

        interpolate_colors(&colors, t)
    }
}

/// Ice/cold palette.
///
/// Colors: black → dark blue → blue → light blue → cyan → white
/// Creates a frozen/icy appearance.
pub struct IcePalette;

impl Palette for IcePalette {
    fn name(&self) -> &str {
        "Ice"
    }

    fn color(&self, t: f32) -> Color32 {
        let colors: [(f32, f32, f32); 6] = [
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 0.5),
            (0.0, 0.0, 1.0),
            (0.0, 0.5, 1.0),
            (0.5, 1.0, 1.0),
            (1.0, 1.0, 1.0),
        ];

        interpolate_colors(&colors, t)
    }
}

/// Simple grayscale palette.
///
/// Maps t directly to brightness from black to white.
/// Useful for analyzing fractal structure without color distraction.
pub struct GrayscalePalette;

impl Palette for GrayscalePalette {
    fn name(&self) -> &str {
        "Grayscale"
    }

    fn color(&self, t: f32) -> Color32 {
        let v = (t * 255.0) as u8;
        Color32::from_rgb(v, v, v)
    }
}

/// Psychedelic cycling palette.
///
/// Uses HSV color space with full saturation and varying hue.
/// Supports hue rotation via offset parameter for animated effects.
pub struct PsychedelicPalette;

impl Palette for PsychedelicPalette {
    fn name(&self) -> &str {
        "Psychedelic"
    }

    fn color(&self, t: f32) -> Color32 {
        let (r, g, b) = hsv_to_rgb(t, 1.0, 0.5);
        Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

/// Interpolates between a list of RGB colors.
///
/// t should be in range [0.0, 1.0]. Colors are evenly spaced across this range.
fn interpolate_colors(colors: &[(f32, f32, f32)], t: f32) -> Color32 {
    let n = (colors.len() - 1) as f32;
    let idx = (t * n).clamp(0.0, n);
    let i = idx.floor() as usize;
    let f = idx.fract();

    if i >= colors.len() - 1 {
        let last = colors[colors.len() - 1];
        return Color32::from_rgb(
            (last.0 * 255.0) as u8,
            (last.1 * 255.0) as u8,
            (last.2 * 255.0) as u8,
        );
    }

    let (r1, g1, b1) = colors[i];
    let (r2, g2, b2) = colors[i + 1];

    Color32::from_rgb(
        ((r1 + (r2 - r1) * f) * 255.0) as u8,
        ((g1 + (g2 - g1) * f) * 255.0) as u8,
        ((b1 + (b2 - b1) * f) * 255.0) as u8,
    )
}

/// Converts HSV color to RGB.
///
/// h: hue in range [0.0, 1.0] (0=red, 1/3=green, 2/3=blue)
/// s: saturation in range [0.0, 1.0]
/// v: value/brightness in range [0.0, 1.0]
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

/// Gets a color from the specified palette.
///
/// For Psychedelic palette, offset is added to t for hue rotation.
pub fn get_color(palette_type: PaletteType, t: f32, offset: f32) -> Color32 {
    let adjusted_t = if matches!(palette_type, PaletteType::Psychedelic) {
        (t + offset) % 1.0
    } else {
        t
    };

    match palette_type {
        PaletteType::Classic => CLASSIC_PALETTE
            .get_or_init(|| ClassicPalette)
            .color(adjusted_t),
        PaletteType::Fire => FIRE_PALETTE.get_or_init(|| FirePalette).color(adjusted_t),
        PaletteType::Ice => ICE_PALETTE.get_or_init(|| IcePalette).color(adjusted_t),
        PaletteType::Grayscale => GRAYSCALE_PALETTE
            .get_or_init(|| GrayscalePalette)
            .color(adjusted_t),
        PaletteType::Psychedelic => PSYCHEDELIC_PALETTE
            .get_or_init(|| PsychedelicPalette)
            .color(adjusted_t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classic_palette_endpoints() {
        let p = ClassicPalette;
        // t=0.0 should be black (or near-black)
        let c0 = p.color(0.0);
        assert_eq!(c0.r(), 0);
        assert_eq!(c0.g(), 0);
        assert_eq!(c0.b(), 0);

        // t=1.0 should be white
        let c1 = p.color(1.0);
        assert_eq!(c1.r(), 255);
        assert_eq!(c1.g(), 255);
        assert_eq!(c1.b(), 255);
    }

    #[test]
    fn test_fire_palette_endpoints() {
        let p = FirePalette;
        let c0 = p.color(0.0);
        assert_eq!(c0.r(), 0);
        assert_eq!(c0.g(), 0);
        assert_eq!(c0.b(), 0);

        let c1 = p.color(1.0);
        assert_eq!(c1.r(), 255);
        assert_eq!(c1.g(), 255);
        assert_eq!(c1.b(), 255);
    }

    #[test]
    fn test_ice_palette_endpoints() {
        let p = IcePalette;
        let c0 = p.color(0.0);
        assert_eq!(c0.r(), 0);

        let c1 = p.color(1.0);
        assert_eq!(c1.r(), 255);
        assert_eq!(c1.g(), 255);
        assert_eq!(c1.b(), 255);
    }

    #[test]
    fn test_grayscale_palette() {
        let p = GrayscalePalette;
        let c0 = p.color(0.0);
        assert_eq!(c0.r(), 0);
        assert_eq!(c0.g(), 0);
        assert_eq!(c0.b(), 0);

        let c1 = p.color(1.0);
        assert_eq!(c1.r(), 255);
        assert_eq!(c1.g(), 255);
        assert_eq!(c1.b(), 255);

        // Midpoint should be ~127
        let mid = p.color(0.5);
        assert!((mid.r() as i32 - 127).abs() <= 1);
    }

    #[test]
    fn test_psychedelic_palette_not_black() {
        let p = PsychedelicPalette;
        // At t=0 with HSV: h=0 (red), s=1, v=0.5 -> should be dark red
        let c = p.color(0.0);
        assert!(c.r() > 0 || c.g() > 0 || c.b() > 0, "Should not be black");
    }

    #[test]
    fn test_interpolate_monotonic() {
        // Classic palette should produce smoothly varying colors
        let p = ClassicPalette;
        let mut prev_total = 0u32;
        let mut non_monotonic_count = 0;

        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let c = p.color(t);
            let total = c.r() as u32 + c.g() as u32 + c.b() as u32;
            // Allow some non-monotonicity (color space traversal)
            // but overall brightness should trend upward
            if total < prev_total && prev_total - total > 50 {
                non_monotonic_count += 1;
            }
            prev_total = total;
        }
        // There should be at most a couple of dips in the rainbow
        assert!(
            non_monotonic_count <= 3,
            "Too many large brightness drops: {}",
            non_monotonic_count
        );
    }

    #[test]
    fn test_get_color_all_palettes() {
        // Every palette should return a valid color for any t in [0, 1]
        for palette in [
            PaletteType::Classic,
            PaletteType::Fire,
            PaletteType::Ice,
            PaletteType::Grayscale,
            PaletteType::Psychedelic,
        ] {
            for i in 0..=10 {
                let t = i as f32 / 10.0;
                let c = get_color(palette, t, 0.0);
                // Just verify we get a color without panicking
                let _ = c.r();
                let _ = c.g();
                let _ = c.b();
            }
        }
    }

    #[test]
    fn test_psychedelic_offset() {
        // Different offsets should produce different colors
        let c1 = get_color(PaletteType::Psychedelic, 0.5, 0.0);
        let c2 = get_color(PaletteType::Psychedelic, 0.5, 0.5);
        assert_ne!(c1, c2, "Different offsets should produce different colors");
    }

    #[test]
    fn test_hsv_to_rgb_red() {
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!(b.abs() < 0.01);
    }

    #[test]
    fn test_hsv_to_rgb_green() {
        let (r, g, b) = hsv_to_rgb(1.0 / 3.0, 1.0, 1.0);
        assert!(r.abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!(b.abs() < 0.01);
    }

    #[test]
    fn test_hsv_to_rgb_blue() {
        let (r, g, b) = hsv_to_rgb(2.0 / 3.0, 1.0, 1.0);
        assert!(r.abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hsv_to_rgb_white() {
        let (r, g, b) = hsv_to_rgb(0.0, 0.0, 1.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }
}
