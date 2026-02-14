use eframe::egui::Color32;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

use crate::palette::{get_color, PaletteType};

/// Available color processor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorProcessorType {
    #[default]
    Palette,
    Smooth,
    OrbitTrapReal,
    OrbitTrapImag,
    OrbitTrapOrigin,
}

impl ColorProcessorType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ColorProcessorType::Palette => "Standard Palette",
            ColorProcessorType::Smooth => "Smooth Coloring",
            ColorProcessorType::OrbitTrapReal => "Orbit Trap (Real Axis)",
            ColorProcessorType::OrbitTrapImag => "Orbit Trap (Imaginary Axis)",
            ColorProcessorType::OrbitTrapOrigin => "Orbit Trap (Origin)",
        }
    }

    pub fn create_processor(&self) -> Box<dyn ColorProcessor> {
        match self {
            ColorProcessorType::Palette => Box::new(PaletteProcessor),
            ColorProcessorType::Smooth => Box::new(SmoothColoring::new(true)),
            ColorProcessorType::OrbitTrapReal => {
                Box::new(OrbitTrapProcessor::new(TrapType::RealAxis, 0.1))
            }
            ColorProcessorType::OrbitTrapImag => {
                Box::new(OrbitTrapProcessor::new(TrapType::ImagAxis, 0.1))
            }
            ColorProcessorType::OrbitTrapOrigin => {
                Box::new(OrbitTrapProcessor::new(TrapType::Origin, 0.5))
            }
        }
    }
}

/// Context passed to color processors during rendering
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ColorContext {
    pub max_iterations: u32,
    pub palette_type: PaletteType,
    pub palette_offset: f32,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl ColorContext {
    pub fn new(
        max_iterations: u32,
        palette_type: PaletteType,
        palette_offset: f32,
        screen_width: u32,
        screen_height: u32,
    ) -> Self {
        Self {
            max_iterations,
            palette_type,
            palette_offset,
            screen_width,
            screen_height,
        }
    }
}

/// Result of fractal computation including iteration count and orbit data
#[derive(Debug, Clone, Copy)]
pub struct FractalResult {
    pub iterations: u32,
    pub escaped: bool,
    pub final_z: Option<Complex64>,
    pub orbit_data: OrbitData,
}

impl FractalResult {
    pub fn inside_set(iterations: u32) -> Self {
        Self {
            iterations,
            escaped: false,
            final_z: None,
            orbit_data: OrbitData::default(),
        }
    }

    pub fn escaped(iterations: u32, final_z: Complex64, orbit_data: OrbitData) -> Self {
        Self {
            iterations,
            escaped: true,
            final_z: Some(final_z),
            orbit_data,
        }
    }
}

/// Data collected during orbit computation
#[derive(Debug, Clone, Copy, Default)]
pub struct OrbitData {
    pub min_real: f64,
    pub max_real: f64,
    pub min_imag: f64,
    pub max_imag: f64,
    pub min_distance_to_origin: f64,
    pub min_distance_to_real_axis: f64,
    pub min_distance_to_imag_axis: f64,
}

impl OrbitData {
    pub fn new() -> Self {
        Self {
            min_real: f64::INFINITY,
            max_real: f64::NEG_INFINITY,
            min_imag: f64::INFINITY,
            max_imag: f64::NEG_INFINITY,
            min_distance_to_origin: f64::INFINITY,
            min_distance_to_real_axis: f64::INFINITY,
            min_distance_to_imag_axis: f64::INFINITY,
        }
    }

    pub fn update(&mut self, z: Complex64) {
        self.min_real = self.min_real.min(z.re);
        self.max_real = self.max_real.max(z.re);
        self.min_imag = self.min_imag.min(z.im);
        self.max_imag = self.max_imag.max(z.im);
        self.min_distance_to_origin = self.min_distance_to_origin.min(z.norm());
        self.min_distance_to_real_axis = self.min_distance_to_real_axis.min(z.im.abs());
        self.min_distance_to_imag_axis = self.min_distance_to_imag_axis.min(z.re.abs());
    }
}

/// Trait for color processing strategies
pub trait ColorProcessor: Send + Sync {
    /// Process fractal result into a color
    fn process(&self, result: &FractalResult, context: &ColorContext) -> Color32;

    /// Get the name of this processor
    fn name(&self) -> &str;

    /// Clone this processor into a Box
    fn clone_box(&self) -> Box<dyn ColorProcessor>;
}

impl Clone for Box<dyn ColorProcessor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Simple palette-based coloring (current behavior)
#[derive(Clone, Copy)]
pub struct PaletteProcessor;

impl ColorProcessor for PaletteProcessor {
    fn process(&self, result: &FractalResult, context: &ColorContext) -> Color32 {
        if !result.escaped {
            Color32::BLACK
        } else {
            let t = result.iterations as f32 / context.max_iterations as f32;
            get_color(context.palette_type, t, context.palette_offset)
        }
    }

    fn name(&self) -> &str {
        "Palette"
    }

    fn clone_box(&self) -> Box<dyn ColorProcessor> {
        Box::new(*self)
    }
}

/// Smooth coloring using continuous iteration count
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct SmoothColoring {
    pub smoothing_enabled: bool,
}

#[allow(dead_code)]
impl SmoothColoring {
    pub fn new(enabled: bool) -> Self {
        Self {
            smoothing_enabled: enabled,
        }
    }

    /// Calculate smooth iteration count
    /// Uses the formula: n - log(log(|z|)) / log(2)
    fn smooth_iterations(&self, result: &FractalResult, _context: &ColorContext) -> f32 {
        if !result.escaped || result.final_z.is_none() {
            return result.iterations as f32;
        }

        let Some(z) = result.final_z else {
            return result.iterations as f32;
        };

        // Use norm_sqr() to avoid sqrt, then adjust: log(sqrt(x)) = 0.5 * log(x)
        let z_norm_sq = z.norm_sqr();
        let log_z = 0.5 * z_norm_sq.ln();
        let log_log_z = log_z.ln();

        if !log_log_z.is_finite() {
            return result.iterations as f32;
        }

        // Standard smooth iteration formula: nu = n - log(log|z|) / log(2)
        let nu = result.iterations as f64 - log_log_z / std::f64::consts::LN_2;
        nu as f32
    }
}

impl ColorProcessor for SmoothColoring {
    fn process(&self, result: &FractalResult, context: &ColorContext) -> Color32 {
        if !result.escaped {
            return Color32::BLACK;
        }

        let t = if self.smoothing_enabled {
            let smooth_iter = self.smooth_iterations(result, context);
            (smooth_iter / context.max_iterations as f32).fract()
        } else {
            result.iterations as f32 / context.max_iterations as f32
        };

        get_color(context.palette_type, t, context.palette_offset)
    }

    fn name(&self) -> &str {
        if self.smoothing_enabled {
            "Smooth Coloring"
        } else {
            "Discrete Coloring"
        }
    }

    fn clone_box(&self) -> Box<dyn ColorProcessor> {
        Box::new(*self)
    }
}

/// Orbit trap coloring (for Pickover Stalk, etc.)
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct OrbitTrapProcessor {
    pub trap_type: TrapType,
    pub threshold: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TrapType {
    RealAxis, // Distance to real axis
    ImagAxis, // Distance to imaginary axis
    Origin,   // Distance to origin
    Cross,    // Both axes
}

#[allow(dead_code)]
impl OrbitTrapProcessor {
    pub fn new(trap_type: TrapType, threshold: f64) -> Self {
        Self {
            trap_type,
            threshold,
        }
    }

    fn get_trap_value(&self, data: &OrbitData) -> f64 {
        match self.trap_type {
            TrapType::RealAxis => data.min_distance_to_real_axis,
            TrapType::ImagAxis => data.min_distance_to_imag_axis,
            TrapType::Origin => data.min_distance_to_origin,
            TrapType::Cross => data
                .min_distance_to_real_axis
                .min(data.min_distance_to_imag_axis),
        }
    }
}

impl ColorProcessor for OrbitTrapProcessor {
    fn process(&self, result: &FractalResult, context: &ColorContext) -> Color32 {
        if !result.escaped {
            return Color32::BLACK;
        }

        let trap_value = self.get_trap_value(&result.orbit_data);

        // Normalize trap value to 0-1 range
        // Smaller distances = closer to trap = brighter
        let t = if self.threshold > 0.0 {
            (1.0 - (trap_value / self.threshold).min(1.0)) as f32
        } else {
            0.0
        };

        // Mix with palette based on iterations
        let iter_t = result.iterations as f32 / context.max_iterations as f32;
        let mixed_t = t * 0.7 + iter_t * 0.3;

        get_color(context.palette_type, mixed_t, context.palette_offset)
    }

    fn name(&self) -> &str {
        match self.trap_type {
            TrapType::RealAxis => "Real Axis Trap",
            TrapType::ImagAxis => "Imaginary Axis Trap",
            TrapType::Origin => "Origin Trap",
            TrapType::Cross => "Cross Trap",
        }
    }

    fn clone_box(&self) -> Box<dyn ColorProcessor> {
        Box::new(*self)
    }
}

/// Chain multiple processors together
#[allow(dead_code)]
pub struct ChainProcessor {
    processors: Vec<Box<dyn ColorProcessor>>,
}

#[allow(dead_code)]
impl ChainProcessor {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add(mut self, processor: Box<dyn ColorProcessor>) -> Self {
        self.processors.push(processor);
        self
    }
}

impl ColorProcessor for ChainProcessor {
    fn process(&self, result: &FractalResult, context: &ColorContext) -> Color32 {
        // For now, just use the last processor's result
        // In a more advanced implementation, this could blend results
        self.processors
            .last()
            .map(|p| p.process(result, context))
            .unwrap_or(Color32::BLACK)
    }

    fn name(&self) -> &str {
        "Chain"
    }

    fn clone_box(&self) -> Box<dyn ColorProcessor> {
        // Clone all processors in the chain
        let cloned: Vec<_> = self.processors.iter().map(|p| p.clone_box()).collect();
        Box::new(ChainProcessor { processors: cloned })
    }
}

/// Color pipeline that manages the active processor
pub struct ColorPipeline {
    processor: Box<dyn ColorProcessor>,
}

impl Clone for ColorPipeline {
    fn clone(&self) -> Self {
        Self {
            processor: self.processor.clone_box(),
        }
    }
}

impl Default for ColorPipeline {
    fn default() -> Self {
        Self {
            processor: Box::new(PaletteProcessor),
        }
    }
}

impl ColorPipeline {
    pub fn from_type(processor_type: ColorProcessorType) -> Self {
        Self {
            processor: processor_type.create_processor(),
        }
    }
}

#[allow(dead_code)]
impl ColorPipeline {
    pub fn new(processor: Box<dyn ColorProcessor>) -> Self {
        Self { processor }
    }

    pub fn set_processor(&mut self, processor: Box<dyn ColorProcessor>) {
        self.processor = processor;
    }

    pub fn process(&self, result: &FractalResult, context: &ColorContext) -> Color32 {
        self.processor.process(result, context)
    }

    pub fn processor_name(&self) -> &str {
        self.processor.name()
    }
}

/// Helper function to compute orbit data during fractal iteration
#[allow(dead_code)]
pub fn compute_with_orbit<F>(mut f: F, cx: f64, cy: f64, max_iter: u32) -> FractalResult
where
    F: FnMut(Complex64, Complex64) -> Complex64,
{
    let mut z = Complex64::new(0.0, 0.0);
    let c = Complex64::new(cx, cy);
    let mut orbit_data = OrbitData::new();

    for i in 0..max_iter {
        if z.norm_sqr() > 4.0 {
            return FractalResult::escaped(i, z, orbit_data);
        }

        z = f(z, c);
        orbit_data.update(z);
    }

    FractalResult::inside_set(max_iter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_processor_inside() {
        let processor = PaletteProcessor;
        let context = ColorContext::new(100, PaletteType::Classic, 0.0, 100, 100);
        let result = FractalResult::inside_set(100);

        let color = processor.process(&result, &context);
        assert_eq!(color, Color32::BLACK);
    }

    #[test]
    fn test_palette_processor_escaped() {
        let processor = PaletteProcessor;
        let context = ColorContext::new(100, PaletteType::Classic, 0.0, 100, 100);
        let result = FractalResult::escaped(50, Complex64::new(2.0, 0.0), OrbitData::new());

        let color = processor.process(&result, &context);
        // Should not be black for escaped points
        assert_ne!(color, Color32::BLACK);
    }

    #[test]
    fn test_smooth_coloring() {
        let processor = SmoothColoring::new(true);
        let context = ColorContext::new(100, PaletteType::Classic, 0.0, 100, 100);
        let result = FractalResult::escaped(50, Complex64::new(2.5, 0.0), OrbitData::new());

        let color = processor.process(&result, &context);
        assert_ne!(color, Color32::BLACK);
    }

    #[test]
    fn test_orbit_trap_processor() {
        let processor = OrbitTrapProcessor::new(TrapType::RealAxis, 0.1);
        let context = ColorContext::new(100, PaletteType::Classic, 0.0, 100, 100);

        let mut orbit_data = OrbitData::new();
        orbit_data.min_distance_to_real_axis = 0.05; // Close to real axis

        let result = FractalResult {
            iterations: 50,
            escaped: true,
            final_z: Some(Complex64::new(1.0, 0.05)),
            orbit_data,
        };

        let color = processor.process(&result, &context);
        assert_ne!(color, Color32::BLACK);
    }

    #[test]
    fn test_orbit_data_update() {
        let mut data = OrbitData::new();
        data.update(Complex64::new(1.0, 2.0));
        data.update(Complex64::new(-1.0, -2.0));

        assert_eq!(data.min_real, -1.0);
        assert_eq!(data.max_real, 1.0);
        assert_eq!(data.min_imag, -2.0);
        assert_eq!(data.max_imag, 2.0);
    }

    #[test]
    fn test_color_pipeline() {
        let mut pipeline = ColorPipeline::default();
        assert_eq!(pipeline.processor_name(), "Palette");

        pipeline.set_processor(Box::new(SmoothColoring::new(true)));
        assert_eq!(pipeline.processor_name(), "Smooth Coloring");
    }
}
