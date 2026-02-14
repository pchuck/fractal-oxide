use num_complex::Complex64;

/// Manages the view transformation between screen and fractal coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Center of the view in fractal coordinates
    center: Complex64,
    /// Zoom level (1.0 = default view showing ~4 units)
    zoom: f64,
    /// Aspect ratio (width / height)
    aspect_ratio: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center: Complex64::new(-0.5, 0.0),
            zoom: 1.0,
            aspect_ratio: 1.0,
        }
    }
}

#[allow(dead_code)]
impl Viewport {
    /// Create a new viewport with the given center and zoom
    pub fn new(center_x: f64, center_y: f64, zoom: f64) -> Self {
        Self {
            center: Complex64::new(center_x, center_y),
            zoom,
            aspect_ratio: 1.0,
        }
    }

    /// Create a viewport from a view state
    pub fn from_view(center_x: f64, center_y: f64, zoom: f64, width: u32, height: u32) -> Self {
        Self {
            center: Complex64::new(center_x, center_y),
            zoom,
            aspect_ratio: width as f64 / height as f64,
        }
    }

    /// Set the aspect ratio based on screen dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        if height == 0 {
            return; // Avoid division by zero
        }
        self.aspect_ratio = width as f64 / height as f64;
    }

    /// Get the center coordinates
    pub fn center(&self) -> (f64, f64) {
        (self.center.re, self.center.im)
    }

    /// Get the zoom level
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Get the aspect ratio
    pub fn aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }

    /// Convert screen coordinates to fractal coordinates
    pub fn screen_to_world(&self, x: u32, y: u32, width: u32, height: u32) -> Complex64 {
        if width == 0 || height == 0 || self.zoom == 0.0 {
            return self.center; // Return center as fallback
        }
        let uv_x = x as f64 / width as f64;
        let uv_y = y as f64 / height as f64;

        let world_x = self.center.re + (uv_x - 0.5) * 4.0 * self.aspect_ratio / self.zoom;
        let world_y = self.center.im - (uv_y - 0.5) * 4.0 / self.zoom;

        Complex64::new(world_x, world_y)
    }

    /// Convert world coordinates to screen coordinates (for minimap, etc.)
    pub fn world_to_screen(&self, world: Complex64, width: u32, height: u32) -> (i32, i32) {
        if self.aspect_ratio == 0.0 || self.zoom == 0.0 {
            return (0, 0); // Return origin as fallback
        }
        let dx = (world.re - self.center.re) * self.zoom / (4.0 * self.aspect_ratio);
        let dy = -(world.im - self.center.im) * self.zoom / 4.0;

        let screen_x = ((dx + 0.5) * width as f64) as i32;
        let screen_y = ((dy + 0.5) * height as f64) as i32;

        (screen_x, screen_y)
    }

    /// Pan the view by the given amount in screen pixels
    /// Returns the actual pan amount in world coordinates
    pub fn pan(&mut self, dx_pixels: f64, dy_pixels: f64, screen_size: f64) -> (f64, f64) {
        // Calculate world units per pixel
        if screen_size <= 0.0 || self.zoom <= 0.0 {
            return (0.0, 0.0);
        }
        let world_per_pixel = 4.0 / (screen_size * self.zoom);

        let world_dx = dx_pixels * world_per_pixel * self.aspect_ratio;
        let world_dy = -dy_pixels * world_per_pixel; // Invert Y for screen coords

        self.center.re += world_dx;
        self.center.im += world_dy;

        (world_dx, world_dy)
    }

    /// Pan by a fixed amount (for keyboard navigation)
    pub fn pan_fixed(&mut self, dx: f64, dy: f64) {
        if self.zoom <= 0.0 {
            return;
        }
        let pan_amount = 0.5 / self.zoom;
        self.center.re += dx * pan_amount;
        self.center.im += dy * pan_amount;
    }

    /// Zoom by a factor, optionally keeping a point stationary
    pub fn zoom_by(&mut self, factor: f64, focus: Option<(u32, u32)>, width: u32, height: u32) {
        const MIN_ZOOM: f64 = 1e-15;
        const MAX_ZOOM: f64 = 1e15;

        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let actual_factor = new_zoom / self.zoom;

        if let Some((fx, fy)) = focus {
            // Zoom towards focus point
            let focus_world = self.screen_to_world(fx, fy, width, height);

            // Calculate offset from center
            let offset_x = focus_world.re - self.center.re;
            let offset_y = focus_world.im - self.center.im;

            // Apply zoom
            self.zoom = new_zoom;

            // Adjust center to keep focus point stationary
            self.center.re = focus_world.re - offset_x / actual_factor;
            self.center.im = focus_world.im - offset_y / actual_factor;
        } else {
            // Simple zoom about center
            self.zoom = new_zoom;
        }
    }

    /// Set zoom level directly
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom;
    }

    /// Get the visible rectangle in world coordinates (for minimap)
    pub fn visible_rect(&self) -> ((f64, f64), (f64, f64)) {
        if self.zoom == 0.0 {
            return (
                (self.center.re, self.center.im),
                (self.center.re, self.center.im),
            );
        }
        let half_width = 2.0 * self.aspect_ratio / self.zoom;
        let half_height = 2.0 / self.zoom;

        let min = (self.center.re - half_width, self.center.im - half_height);
        let max = (self.center.re + half_width, self.center.im + half_height);

        (min, max)
    }

    /// Calculate the pixel shift for pan optimization
    /// Returns (shift_x, shift_y) in pixels for the given pan amount
    pub fn calculate_pixel_shift(&self, dx: f64, dy: f64, width: u32, height: u32) -> (i32, i32) {
        // Fractal pan: dx * 0.5 / zoom
        // Horizontal visible range: 4.0 * aspect / zoom
        // Vertical visible range: 4.0 / zoom
        if self.aspect_ratio == 0.0 {
            return (0, 0);
        }
        let shift_x = (-dx * width as f64 / (8.0 * self.aspect_ratio)) as i32;
        let shift_y = (dy * height as f64 / 8.0) as i32;

        (shift_x, shift_y)
    }

    /// Get the scale in world units per pixel
    pub fn world_units_per_pixel(&self, screen_pixels: f64) -> f64 {
        4.0 / (screen_pixels * self.zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_default() {
        let vp = Viewport::default();
        assert_eq!(vp.center(), (-0.5, 0.0));
        assert_eq!(vp.zoom(), 1.0);
        assert_eq!(vp.aspect_ratio(), 1.0);
    }

    #[test]
    fn test_screen_to_world_center() {
        let mut vp = Viewport::new(0.0, 0.0, 1.0);
        vp.set_dimensions(100, 100);

        // Center of screen should map to center of view
        let world = vp.screen_to_world(50, 50, 100, 100);
        assert!((world.re - 0.0).abs() < 0.001);
        assert!((world.im - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_screen_to_world_corners() {
        let mut vp = Viewport::new(0.0, 0.0, 1.0);
        vp.set_dimensions(100, 100);

        // At zoom 1.0, visible range is 4.0 units
        // Top-left corner (0, 0) should be at (-2.0, 2.0)
        let tl = vp.screen_to_world(0, 0, 100, 100);
        assert!((tl.re - (-2.0)).abs() < 0.1);
        assert!((tl.im - 2.0).abs() < 0.1);

        // Bottom-right corner (99, 99) should be at (2.0, -2.0)
        let br = vp.screen_to_world(99, 99, 100, 100);
        assert!((br.re - 2.0).abs() < 0.1);
        assert!((br.im - (-2.0)).abs() < 0.1);
    }

    #[test]
    fn test_pan() {
        let mut vp = Viewport::new(0.0, 0.0, 1.0);
        vp.set_dimensions(100, 100);

        vp.pan_fixed(1.0, 0.0); // Pan right
        assert!(vp.center().0 > 0.0);
        assert!((vp.center().1 - 0.0).abs() < 0.001);

        vp.pan_fixed(0.0, 1.0); // Pan up
        assert!(vp.center().1 > 0.0);
    }

    #[test]
    fn test_zoom() {
        let mut vp = Viewport::new(0.0, 0.0, 1.0);
        vp.set_dimensions(100, 100);

        vp.zoom_by(2.0, None, 100, 100);
        assert_eq!(vp.zoom(), 2.0);

        // After zooming, visible range should be halved
        let tl = vp.screen_to_world(0, 0, 100, 100);
        assert!((tl.re - (-1.0)).abs() < 0.1);
    }

    #[test]
    fn test_zoom_to_focus() {
        let mut vp = Viewport::new(0.0, 0.0, 1.0);
        vp.set_dimensions(100, 100);

        // Zoom in towards top-right corner
        let focus = vp.screen_to_world(75, 25, 100, 100);
        vp.zoom_by(2.0, Some((75, 25)), 100, 100);

        // The focus point should still map to the same screen coordinates
        let new_screen = vp.world_to_screen(focus, 100, 100);
        assert!((new_screen.0 - 75).abs() <= 1);
        assert!((new_screen.1 - 25).abs() <= 1);
    }

    #[test]
    fn test_visible_rect() {
        let mut vp = Viewport::new(0.0, 0.0, 1.0);
        vp.set_dimensions(100, 100);

        let (min, max) = vp.visible_rect();
        assert!((min.0 - (-2.0)).abs() < 0.1);
        assert!((min.1 - (-2.0)).abs() < 0.1);
        assert!((max.0 - 2.0).abs() < 0.1);
        assert!((max.1 - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_roundtrip() {
        let mut vp = Viewport::new(-0.5, 0.5, 2.0);
        vp.set_dimensions(200, 100);

        // Pick a screen coordinate
        let screen_x = 50;
        let screen_y = 75;

        // Convert to world
        let world = vp.screen_to_world(screen_x, screen_y, 200, 100);

        // Convert back to screen
        let (back_x, back_y) = vp.world_to_screen(world, 200, 100);

        // Should be approximately the same
        assert!((back_x - screen_x as i32).abs() <= 1);
        assert!((back_y - screen_y as i32).abs() <= 1);
    }
}
