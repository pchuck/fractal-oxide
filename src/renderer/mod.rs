use eframe::egui::Color32;
use rayon::prelude::*;

use crate::color_pipeline::{ColorContext, ColorPipeline};
use crate::fractal::Fractal;
use crate::palette::PaletteType;
use crate::FractalViewState;

/// A rectangular region to render
#[derive(Clone, Debug)]
pub struct RenderRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Configuration for rendering operations
#[derive(Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub supersampling: bool,
    pub max_iterations: u32,
    pub palette_type: PaletteType,
    pub palette_offset: f32,
    pub color_pipeline: ColorPipeline,
}

impl RenderConfig {
    /// Get the actual render dimensions (accounting for supersampling)
    pub fn render_dimensions(&self) -> (u32, u32) {
        if self.supersampling {
            (self.width * 2, self.height * 2)
        } else {
            (self.width, self.height)
        }
    }

    /// Create a color context for the current render settings
    fn color_context(&self) -> ColorContext {
        ColorContext::new(
            self.max_iterations,
            self.palette_type,
            self.palette_offset,
            self.width,
            self.height,
        )
    }
}

/// Result of rendering a chunk
#[derive(Debug)]
pub struct ChunkResult {
    pub pixels: Vec<Color32>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// The rendering engine - handles all fractal rendering operations
#[derive(Default)]
pub struct RenderEngine {
    // Buffer for supersampled rendering (2x resolution)
    supersample_buffer: Option<Vec<Color32>>,
    // Buffer for normal rendering
    render_buffer: Option<Vec<Color32>>,
}

impl RenderEngine {
    /// Initialize buffers for a new render
    pub fn start_render(&mut self, config: &RenderConfig) {
        let (render_width, render_height) = config.render_dimensions();
        let buffer_size = (render_width * render_height) as usize;

        if config.supersampling {
            self.supersample_buffer = Some(vec![Color32::BLACK; buffer_size]);
            self.render_buffer = None;
        } else {
            self.render_buffer = Some(vec![Color32::BLACK; buffer_size]);
            self.supersample_buffer = None;
        }
    }

    /// Render a horizontal chunk of the full canvas
    /// Returns true if more work remains, false if complete
    pub fn render_full_chunk(
        &mut self,
        fractal: &dyn Fractal,
        view: &FractalViewState,
        config: &RenderConfig,
        y_start: u32,
        chunk_size: u32,
    ) -> bool {
        let (render_width, render_height) = config.render_dimensions();
        let y_end = (y_start + chunk_size).min(render_height);

        if y_start >= render_height {
            return false;
        }

        let chunk_pixels: Vec<Color32> = (y_start..y_end)
            .into_par_iter()
            .flat_map(|y| {
                (0..render_width)
                    .map(|x| {
                        compute_pixel(x, y, render_width, render_height, fractal, view, config)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Write to appropriate buffer
        let buffer = if config.supersampling {
            self.supersample_buffer.as_mut()
        } else {
            self.render_buffer.as_mut()
        };

        if let Some(buf) = buffer {
            let start_idx = y_start as usize * render_width as usize;
            let chunk_len = (y_end - y_start) as usize * render_width as usize;
            buf[start_idx..start_idx + chunk_len].copy_from_slice(&chunk_pixels);
        }

        true
    }

    /// Render a region (for pan optimization)
    /// Returns the rendered pixels for the region
    pub fn render_region(
        &self,
        region: &RenderRegion,
        fractal: &dyn Fractal,
        view: &FractalViewState,
        config: &RenderConfig,
        y_start: u32,
        chunk_size: u32,
    ) -> Option<ChunkResult> {
        let display_width = config.width;
        let display_height = config.height;
        let y_end = (y_start + chunk_size).min(region.height);

        if y_start >= region.height {
            return None;
        }

        let region_pixels: Vec<Color32> = (y_start..y_end)
            .into_par_iter()
            .flat_map(|dy| {
                let y = region.y + dy;
                (region.x..region.x + region.width)
                    .map(|x| {
                        if config.supersampling {
                            // When supersampling for regions, we still render at 2x
                            // but map back to display coordinates
                            compute_pixel_supersampled(
                                x,
                                y,
                                display_width,
                                display_height,
                                fractal,
                                view,
                                config,
                            )
                        } else {
                            compute_pixel(
                                x,
                                y,
                                display_width,
                                display_height,
                                fractal,
                                view,
                                config,
                            )
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Some(ChunkResult {
            pixels: region_pixels,
            x: region.x,
            y: region.y + y_start,
            width: region.width,
            height: y_end - y_start,
        })
    }

    /// Finalize rendering and return the final pixel buffer
    /// For supersampling, this downsamples from 2x to 1x
    pub fn finalize(&mut self, config: &RenderConfig) -> Option<Vec<Color32>> {
        if config.supersampling {
            self.supersample_buffer.take().map(|pixels| {
                let (render_width, render_height) = config.render_dimensions();
                downsample_2x(&pixels, render_width, render_height)
            })
        } else {
            self.render_buffer.take()
        }
    }

    /// Calculate regions that need rendering after a pan operation
    /// Returns the regions and applies the pixel shift to the image
    pub fn calculate_pan_regions(
        &self,
        image: &mut eframe::egui::ColorImage,
        dx: f64,
        dy: f64,
        _zoom: f64,
    ) -> Vec<RenderRegion> {
        let width = image.width() as u32;
        let height = image.height() as u32;

        // Calculate pixel shift based on fractal pan amount
        // Fractal pan: 0.5 / zoom per keypress
        // Visible range: 4.0 * aspect / zoom horizontal, 4.0 / zoom vertical
        let aspect = width as f64 / height as f64;
        let shift_x = (-dx * width as f64 / (8.0 * aspect)) as i32;
        let shift_y = (dy * height as f64 / 8.0) as i32;

        // Clamp shift values to image dimensions
        let shift_x = shift_x.clamp(-(width as i32), width as i32);
        let shift_y = shift_y.clamp(-(height as i32), height as i32);

        if shift_x == 0 && shift_y == 0 {
            return Vec::new();
        }

        // Shift existing pixels
        let mut new_pixels = vec![Color32::BLACK; image.pixels.len()];

        for y in 0..height {
            for x in 0..width {
                let src_x = x as i32 - shift_x;
                let src_y = y as i32 - shift_y;

                if src_x >= 0 && src_x < width as i32 && src_y >= 0 && src_y < height as i32 {
                    let src_idx = (src_y as usize) * (width as usize) + (src_x as usize);
                    let dst_idx = (y as usize) * (width as usize) + (x as usize);
                    new_pixels[dst_idx] = image.pixels[src_idx];
                }
            }
        }

        image.pixels = new_pixels;

        // Calculate edge regions that need rendering
        let mut regions = Vec::new();

        if shift_x > 0 {
            // Panned left, render left edge
            regions.push(RenderRegion {
                x: 0,
                y: 0,
                width: shift_x as u32,
                height,
            });
        } else if shift_x < 0 {
            // Panned right, render right edge
            let shift_x_abs = (-shift_x) as u32;
            let x = width.saturating_sub(shift_x_abs);
            regions.push(RenderRegion {
                x,
                y: 0,
                width: shift_x_abs.min(width - x),
                height,
            });
        }

        if shift_y > 0 {
            // Panned down, render top edge
            regions.push(RenderRegion {
                x: 0,
                y: 0,
                width,
                height: shift_y as u32,
            });
        } else if shift_y < 0 {
            // Panned up, render bottom edge
            let shift_y_abs = (-shift_y) as u32;
            let y = height.saturating_sub(shift_y_abs);
            regions.push(RenderRegion {
                x: 0,
                y,
                width,
                height: shift_y_abs.min(height - y),
            });
        }

        // Clamp and validate regions
        regions
            .into_iter()
            .filter(|r| r.width > 0 && r.height > 0 && r.x < width && r.y < height)
            .map(|mut r| {
                if r.x + r.width > width {
                    r.width = width - r.x;
                }
                if r.y + r.height > height {
                    r.height = height - r.y;
                }
                r
            })
            .collect()
    }

    /// Render a high-resolution image for export
    #[allow(clippy::too_many_arguments)]
    pub fn render_high_res(
        &self,
        fractal: &dyn Fractal,
        view: &FractalViewState,
        width: u32,
        height: u32,
        max_iter: u32,
        palette_type: PaletteType,
        palette_offset: f32,
        color_pipeline: ColorPipeline,
    ) -> Vec<Color32> {
        let config = RenderConfig {
            width,
            height,
            supersampling: false,
            max_iterations: max_iter,
            palette_type,
            palette_offset,
            color_pipeline,
        };

        (0..height)
            .into_par_iter()
            .flat_map(|y| {
                (0..width)
                    .map(|x| compute_pixel(x, y, width, height, fractal, view, &config))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

/// Convert screen coordinates to fractal coordinates
pub fn screen_to_fractal(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    view: &FractalViewState,
) -> (f64, f64) {
    let aspect = width as f64 / height as f64;
    let uv_x = x as f64 / width as f64;
    let uv_y = y as f64 / height as f64;
    let px = view.center_x + (uv_x - 0.5) * 4.0 * aspect / view.zoom;
    let py = view.center_y - (uv_y - 0.5) * 4.0 / view.zoom;
    (px, py)
}

/// Compute color for a single pixel
fn compute_pixel(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    fractal: &dyn Fractal,
    view: &FractalViewState,
    config: &RenderConfig,
) -> Color32 {
    let (px, py) = screen_to_fractal(x, y, width, height, view);
    let result = fractal.compute_full(px, py, config.max_iterations);

    let context = config.color_context();
    config.color_pipeline.process(&result, &context)
}

/// Compute pixel with 2x2 supersampling and averaging
fn compute_pixel_supersampled(
    x: u32,
    y: u32,
    display_width: u32,
    display_height: u32,
    fractal: &dyn Fractal,
    view: &FractalViewState,
    config: &RenderConfig,
) -> Color32 {
    let render_width = display_width * 2;
    let render_height = display_height * 2;

    let mut r_sum = 0u32;
    let mut g_sum = 0u32;
    let mut b_sum = 0u32;

    let context = config.color_context();

    for sy in 0..2 {
        for sx in 0..2 {
            let sx_coord = x * 2 + sx;
            let sy_coord = y * 2 + sy;

            let (px, py) = screen_to_fractal(sx_coord, sy_coord, render_width, render_height, view);
            let result = fractal.compute_full(px, py, config.max_iterations);
            let color = config.color_pipeline.process(&result, &context);

            r_sum += color.r() as u32;
            g_sum += color.g() as u32;
            b_sum += color.b() as u32;
        }
    }

    Color32::from_rgb((r_sum / 4) as u8, (g_sum / 4) as u8, (b_sum / 4) as u8)
}

/// Downsample 2x image to 1x using box filter
fn downsample_2x(pixels: &[Color32], width: u32, height: u32) -> Vec<Color32> {
    let display_width = width / 2;
    let display_height = height / 2;
    let mut downsampled = vec![Color32::BLACK; (display_width * display_height) as usize];

    for y in 0..display_height {
        for x in 0..display_width {
            let x0 = (x * 2) as usize;
            let x1 = (x * 2 + 1) as usize;
            let y0 = (y * 2) as usize;
            let y1 = (y * 2 + 1) as usize;

            let idx00 = y0 * (width as usize) + x0;
            let idx01 = y0 * (width as usize) + x1;
            let idx10 = y1 * (width as usize) + x0;
            let idx11 = y1 * (width as usize) + x1;

            let c00 = pixels[idx00];
            let c01 = pixels[idx01];
            let c10 = pixels[idx10];
            let c11 = pixels[idx11];

            let r = ((c00.r() as u16 + c01.r() as u16 + c10.r() as u16 + c11.r() as u16) / 4) as u8;
            let g = ((c00.g() as u16 + c01.g() as u16 + c10.g() as u16 + c11.g() as u16) / 4) as u8;
            let b = ((c00.b() as u16 + c01.b() as u16 + c10.b() as u16 + c11.b() as u16) / 4) as u8;

            downsampled[(y * display_width + x) as usize] = Color32::from_rgb(r, g, b);
        }
    }

    downsampled
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_view() -> FractalViewState {
        FractalViewState {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            max_iterations: 100,
            fractal_params: HashMap::new(),
            color_processor_type: crate::color_pipeline::ColorProcessorType::default(),
            palette_type: PaletteType::Classic,
        }
    }

    #[test]
    fn test_render_config_dimensions() {
        let config_normal = RenderConfig {
            width: 100,
            height: 100,
            supersampling: false,
            max_iterations: 100,
            palette_type: PaletteType::Classic,
            palette_offset: 0.0,
            color_pipeline: ColorPipeline::default(),
        };
        assert_eq!(config_normal.render_dimensions(), (100, 100));

        let config_ss = RenderConfig {
            width: 100,
            height: 100,
            supersampling: true,
            max_iterations: 100,
            palette_type: PaletteType::Classic,
            palette_offset: 0.0,
            color_pipeline: ColorPipeline::default(),
        };
        assert_eq!(config_ss.render_dimensions(), (200, 200));
    }

    #[test]
    fn test_screen_to_fractal_center() {
        let view = test_view();
        let (px, py) = screen_to_fractal(50, 50, 100, 100, &view);
        assert!((px - 0.0).abs() < 0.001, "Center x should be 0");
        assert!((py - 0.0).abs() < 0.001, "Center y should be 0");
    }

    #[test]
    fn test_downsample_2x() {
        let pixels: Vec<Color32> = (0..16)
            .map(|i| {
                if i % 2 == 0 {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                }
            })
            .collect();

        let downsampled = downsample_2x(&pixels, 4, 4);

        assert_eq!(downsampled.len(), 4);

        for pixel in &downsampled {
            assert_eq!(pixel.r(), 127);
            assert_eq!(pixel.g(), 127);
            assert_eq!(pixel.b(), 127);
        }
    }

    #[test]
    fn test_pan_regions_left() {
        let engine = RenderEngine::default();
        let mut image = eframe::egui::ColorImage {
            size: [100, 100],
            pixels: vec![Color32::BLACK; 10000],
        };

        // Pan left (dx > 0 means moving view right, so pixels shift left)
        let regions = engine.calculate_pan_regions(&mut image, 1.0, 0.0, 1.0);

        assert!(!regions.is_empty());
        // Should have a region on the right edge
        assert!(regions.iter().any(|r| r.x > 0));
    }

    #[test]
    fn test_render_region_clamping() {
        let engine = RenderEngine::default();
        let mut image = eframe::egui::ColorImage {
            size: [100, 100],
            pixels: vec![Color32::BLACK; 10000],
        };

        // Large pan values should still produce valid regions
        let regions = engine.calculate_pan_regions(&mut image, 10.0, 10.0, 1.0);

        for region in &regions {
            assert!(region.x < 100);
            assert!(region.y < 100);
            assert!(region.x + region.width <= 100);
            assert!(region.y + region.height <= 100);
        }
    }
}
