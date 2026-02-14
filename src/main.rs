use eframe::egui;
use image::{ImageBuffer, Rgb};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

mod color_pipeline;
mod command;
mod fractal;
mod palette;
mod renderer;
mod ui;
mod viewport;

use command::{AppState, CommandHistory, ViewCommand};
use fractal::{registry::FractalRegistry, Fractal, FractalType};
use palette::PaletteType;
use renderer::{RenderConfig, RenderEngine, RenderRegion};
use ui::{FractalControls, RenderStatus};
use viewport::Viewport;

// Application-wide constants
const DEFAULT_WINDOW_WIDTH: f32 = 1200.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;
const DEFAULT_ITERATIONS: u32 = 200;
const MAX_ITERATIONS_CAP: u32 = 2000;
const ADAPTIVE_ITER_COEFFICIENT: f64 = 50.0;
const UNDO_HISTORY_CAPACITY: usize = 50;
const STATUS_TIMEOUT_SECS: f64 = 3.0;
const DRAG_THRESHOLD_PX: f32 = 10.0;
const RENDER_DELAY_FRAMES: u32 = 2;
const MINIMAP_SIZE: usize = 150;
const MINIMAP_MAX_ITER: u32 = 50;
const MINIMAP_MAP_RANGE: f64 = 4.0;
const CONTROL_PANEL_WIDTH: f32 = 280.0;
const BOOKMARK_SCROLL_HEIGHT: f32 = 150.0;
const ZOOM_KEYBOARD_FACTOR: f64 = 1.5;
const PAN_AMOUNT_BASE: f64 = 0.5;
const SCROLL_ZOOM_SENSITIVITY: f64 = 0.01;
const SCROLL_DEADZONE: f32 = 0.1;
const ABOUT_IMAGE_PATH: &str = "images/mandelbrot_grayscale_904x784.png";
const ABOUT_IMAGE_DISPLAY_WIDTH: f32 = 452.0;
const ABOUT_IMAGE_DISPLAY_HEIGHT: f32 = 392.0;

/// Application configuration for persistence
#[derive(Serialize, Deserialize, Clone)]
struct AppConfig {
    window_width: f32,
    window_height: f32,
    default_iterations: u32,
    default_fractal: FractalType,
    default_palette: PaletteType,
    supersampling_enabled: bool,
    adaptive_iterations: bool,
    bookmarks: Vec<Bookmark>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            default_iterations: DEFAULT_ITERATIONS,
            default_fractal: FractalType::Mandelbrot,
            default_palette: PaletteType::Classic,
            supersampling_enabled: false,
            adaptive_iterations: false,
            bookmarks: Vec::new(),
        }
    }
}

impl AppConfig {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("fractal-oxide").join("config.json"))
    }

    fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&contents) {
                    return config;
                }
            }
        }
        Self::default()
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not determine config directory")?;
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, json).map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }
}

/// Bookmark for saving interesting locations
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Bookmark {
    name: String,
    fractal_type: FractalType,
    center_x: f64,
    center_y: f64,
    zoom: f64,
    max_iterations: u32,
    palette_type: PaletteType,
    #[serde(default)]
    color_processor_type: color_pipeline::ColorProcessorType,
    #[serde(default)]
    fractal_params: HashMap<String, f64>,
}

#[derive(Debug, Clone, Default)]
pub struct FractalViewState {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
    pub max_iterations: u32,
    pub fractal_params: HashMap<String, f64>,
    pub palette_type: PaletteType,
    pub color_processor_type: color_pipeline::ColorProcessorType,
}

/// State related to fractal rendering (engine, config, progress, caches)
struct RenderState {
    engine: RenderEngine,
    config: Option<RenderConfig>,
    needs_render: bool,
    is_rendering: bool,
    render_progress: f32,
    render_start_time: Option<Instant>,
    last_render_time: Option<f64>,
    render_chunk_start: u32,
    /// Partial render regions for pan optimization
    partial_render_regions: Vec<RenderRegion>,
    current_region_index: usize,
    /// Delay rendering by N frames (for zoom preview)
    render_delay: u32,
    /// Cached fractal image pixels
    cached_image: Option<egui::ColorImage>,
    /// GPU texture handle (only recreated when dirty)
    cached_texture: Option<egui::TextureHandle>,
    texture_dirty: bool,
    /// Previous image for zoom preview blitting
    prev_image: Option<egui::ColorImage>,
    cached_width: u32,
    cached_height: u32,
    supersampling_enabled: bool,
    adaptive_iterations: bool,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            engine: RenderEngine::default(),
            config: None,
            needs_render: true,
            is_rendering: false,
            render_progress: 0.0,
            render_start_time: None,
            last_render_time: None,
            render_chunk_start: 0,
            partial_render_regions: Vec::new(),
            current_region_index: 0,
            render_delay: 0,
            cached_image: None,
            cached_texture: None,
            texture_dirty: false,
            prev_image: None,
            cached_width: 0,
            cached_height: 0,
            supersampling_enabled: false,
            adaptive_iterations: false,
        }
    }
}

/// State related to mouse/keyboard interaction
#[derive(Default)]
struct InteractionState {
    drag_start: Option<egui::Pos2>,
    drag_current: Option<egui::Pos2>,
    zoom_preview: Option<ZoomPreview>,
    mouse_fractal_pos: Option<(f64, f64)>,
    status_message: Option<(String, Instant)>,
}

struct FractalApp {
    fractal: Box<dyn Fractal>,
    controls: FractalControls,
    views: HashMap<FractalType, FractalViewState>,
    command_histories: HashMap<FractalType, CommandHistory>,
    render: RenderState,
    interaction: InteractionState,
    bookmarks: Vec<Bookmark>,
    show_bookmark_dialog: bool,
    bookmark_name_input: String,
    minimap_enabled: bool,
    cached_minimap_texture: Option<egui::TextureHandle>,
    minimap_dirty: bool,
    export_scale: u32,
    show_about_dialog: bool,
    cached_about_texture: Option<egui::TextureHandle>,
    fractal_registry: FractalRegistry,
    viewport: Viewport,
    actual_window_width: f32,
    actual_window_height: f32,
}

struct ZoomPreview {
    sel_min: egui::Pos2,
    sel_max: egui::Pos2,
}

impl FractalApp {
    fn new(config: &AppConfig) -> Self {
        let registry = FractalRegistry::default();
        let mut views = HashMap::new();

        // Initialize views from registry metadata
        for ft in registry.all_types() {
            if let Some(metadata) = registry.metadata(ft) {
                // Phoenix has different default iterations
                let max_iter = if ft == FractalType::Phoenix {
                    100
                } else {
                    config.default_iterations
                };
                views.insert(
                    ft,
                    FractalViewState {
                        center_x: metadata.default_center.0,
                        center_y: metadata.default_center.1,
                        zoom: metadata.default_zoom,
                        max_iterations: max_iter,
                        fractal_params: HashMap::new(),
                        palette_type: config.default_palette,
                        color_processor_type: color_pipeline::ColorProcessorType::default(),
                    },
                );
            }
        }

        // Phoenix has different default iterations
        let initial_iterations = if config.default_fractal == FractalType::Phoenix {
            100
        } else {
            config.default_iterations
        };

        let controls = FractalControls {
            fractal_type: config.default_fractal,
            max_iterations: initial_iterations,
            pending_max_iterations: initial_iterations,
            palette_type: config.default_palette,
            ..Default::default()
        };

        let fractal = registry
            .create(config.default_fractal)
            .expect("Default fractal should be registered");

        let render = RenderState {
            supersampling_enabled: config.supersampling_enabled,
            adaptive_iterations: config.adaptive_iterations,
            ..Default::default()
        };

        FractalApp {
            fractal,
            controls,
            views,
            command_histories: registry
                .all_types()
                .into_iter()
                .map(|ft| (ft, CommandHistory::new(UNDO_HISTORY_CAPACITY)))
                .collect(),
            render,
            interaction: InteractionState::default(),
            bookmarks: config.bookmarks.clone(),
            show_bookmark_dialog: false,
            bookmark_name_input: String::new(),
            minimap_enabled: false,
            cached_minimap_texture: None,
            minimap_dirty: true,
            export_scale: 1,
            show_about_dialog: false,
            cached_about_texture: None,
            fractal_registry: registry,
            viewport: Viewport::new(
                config.default_fractal.default_center().0,
                config.default_fractal.default_center().1,
                1.0,
            ),
            actual_window_width: config.window_width,
            actual_window_height: config.window_height,
        }
    }

    /// Helper method to create a fractal using the registry
    fn create_fractal(&self, fractal_type: FractalType) -> Box<dyn Fractal> {
        self.fractal_registry
            .create(fractal_type)
            .expect("Fractal should be registered")
    }

    fn get_view(&self) -> FractalViewState {
        self.views
            .get(&self.controls.fractal_type)
            .cloned()
            .unwrap_or_default()
    }

    fn get_command_history(&mut self) -> &mut CommandHistory {
        self.command_histories
            .entry(self.controls.fractal_type)
            .or_insert_with(|| CommandHistory::new(UNDO_HISTORY_CAPACITY))
    }

    fn set_view(&mut self, view: FractalViewState) {
        self.views.insert(self.controls.fractal_type, view.clone());
        self.viewport = Viewport::from_view(
            view.center_x,
            view.center_y,
            view.zoom,
            self.render.cached_width.max(1),
            self.render.cached_height.max(1),
        );
    }

    fn update_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport.set_dimensions(width, height);
    }

    fn invalidate_cache(&mut self) {
        self.render.needs_render = true;
        self.render.texture_dirty = true;
        self.minimap_dirty = true;
        self.render.partial_render_regions.clear();
        self.render.current_region_index = 0;
    }

    fn calculate_adaptive_iterations(&self, zoom: f64) -> u32 {
        let base_iter = self.controls.max_iterations;
        let zoom_factor = if zoom > 1.0 { zoom.log2() } else { 0.0 };
        let additional = (ADAPTIVE_ITER_COEFFICIENT * zoom_factor) as u32;
        (base_iter + additional).min(MAX_ITERATIONS_CAP)
    }

    fn execute_view_command(&mut self, old_view: &FractalViewState, new_view: &FractalViewState) {
        if (old_view.center_x - new_view.center_x).abs() < 1e-15
            && (old_view.center_y - new_view.center_y).abs() < 1e-15
            && (old_view.zoom - new_view.zoom).abs() < 1e-15
        {
            return;
        }

        let command = Box::new(ViewCommand::from_views(old_view, new_view));
        let mut state = self.to_app_state();
        self.get_command_history().execute(command, &mut state);
        self.apply_app_state(&state);
    }

    fn to_app_state(&self) -> AppState {
        AppState {
            fractal_type: self.controls.fractal_type,
            view: self.get_view(),
            palette_offset: self.controls.palette_offset,
        }
    }

    fn apply_app_state(&mut self, state: &AppState) {
        self.controls.fractal_type = state.fractal_type;
        self.controls.max_iterations = state.view.max_iterations;
        self.controls.pending_max_iterations = state.view.max_iterations;
        self.controls.palette_type = state.view.palette_type;
        self.controls.palette_offset = state.palette_offset;
        self.controls.color_processor_type = state.view.color_processor_type;

        // Update the view and viewport in one place
        self.set_view(state.view.clone());
    }

    fn save_image(&self, scale_factor: u32) -> Result<PathBuf, String> {
        let image = self
            .render
            .cached_image
            .as_ref()
            .ok_or("No image to save - wait for render to complete")?;

        let fractal_name = match self.controls.fractal_type {
            FractalType::Mandelbrot => "mandelbrot",
            FractalType::Julia => "julia",
            FractalType::BurningShip => "burning_ship",
            FractalType::Tricorn => "tricorn",
            FractalType::Celtic => "celtic",
            FractalType::Newton => "newton",
            FractalType::Biomorph => "biomorph",
            FractalType::Phoenix => "phoenix",
            FractalType::Multibrot => "multibrot",
            FractalType::Spider => "spider",
            FractalType::OrbitTrap => "orbit_trap",
            FractalType::PickoverStalk => "pickover_stalk",
        };
        let palette_name = match self.controls.palette_type {
            PaletteType::Classic => "classic",
            PaletteType::Fire => "fire",
            PaletteType::Ice => "ice",
            PaletteType::Grayscale => "grayscale",
            PaletteType::Psychedelic => "psychedelic",
        };

        let base_width = image.width() as u32;
        let base_height = image.height() as u32;
        let width = base_width * scale_factor;
        let height = base_height * scale_factor;

        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

        // If scale_factor is 1, use cached image directly
        if scale_factor == 1 {
            for (i, color) in image.pixels.iter().enumerate() {
                let x = (i % base_width as usize) as u32;
                let y = (i / base_width as usize) as u32;
                img.put_pixel(x, y, Rgb([color.r(), color.g(), color.b()]));
            }
        } else {
            // Render at higher resolution
            self.render_high_res(&mut img, width, height)?;
        }

        let filename = format!(
            "images/{}_{}_{}x{}.png",
            fractal_name, palette_name, width, height
        );
        std::fs::create_dir_all("images")
            .map_err(|e| format!("Failed to create images directory: {}", e))?;
        let path = PathBuf::from(&filename);
        img.save(&path)
            .map_err(|e| format!("Failed to save image: {}", e))?;
        Ok(path)
    }

    fn render_high_res(
        &self,
        buffer: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let view = self.get_view();
        let max_iter = if self.render.adaptive_iterations {
            self.calculate_adaptive_iterations(view.zoom)
        } else {
            self.controls.max_iterations
        };

        let pixels = self.render.engine.render_high_res(
            self.fractal.as_ref(),
            &view,
            width,
            height,
            max_iter,
            self.controls.palette_type,
            self.controls.palette_offset,
            color_pipeline::ColorPipeline::from_type(self.controls.color_processor_type),
        );

        for (i, color) in pixels.iter().enumerate() {
            let x = (i % width as usize) as u32;
            let y = (i / width as usize) as u32;
            buffer.put_pixel(x, y, Rgb([color.r(), color.g(), color.b()]));
        }

        Ok(())
    }

    fn reset_view(&mut self) {
        let (center_x, center_y) = self.controls.fractal_type.default_center();
        let current_max_iter = self.controls.max_iterations;
        let current_palette = self.controls.palette_type;
        let current_params = self
            .views
            .get(&self.controls.fractal_type)
            .map(|v| v.fractal_params.clone())
            .unwrap_or_default();
        let default_view = FractalViewState {
            center_x,
            center_y,
            zoom: 1.0,
            max_iterations: current_max_iter,
            fractal_params: current_params,
            palette_type: current_palette,
            color_processor_type: self.controls.color_processor_type,
        };
        self.set_view(default_view);
    }

    fn reset_settings(&mut self) {
        // Reset everything for current fractal to factory defaults
        let (center_x, center_y) = self.controls.fractal_type.default_center();
        let default_view = FractalViewState {
            center_x,
            center_y,
            zoom: 1.0,
            max_iterations: DEFAULT_ITERATIONS,
            fractal_params: HashMap::new(),
            palette_type: PaletteType::Classic,
            color_processor_type: color_pipeline::ColorProcessorType::default(),
        };
        self.set_view(default_view);

        // Reset controls
        self.controls.max_iterations = DEFAULT_ITERATIONS;
        self.controls.pending_max_iterations = DEFAULT_ITERATIONS;
        self.controls.palette_type = PaletteType::Classic;
        self.controls.pending_palette_offset = 0.0;
        self.controls.palette_offset = 0.0;

        // Reset fractal parameters to defaults
        self.fractal = self.create_fractal(self.controls.fractal_type);
        self.controls.pending_fractal_params.clear();

        self.invalidate_cache();
        self.set_status("Settings reset".to_string());
    }

    /// Zoom centered on a specific screen point (for scroll-wheel zoom)
    fn zoom_at_point(
        &mut self,
        factor: f64,
        screen_x: u32,
        screen_y: u32,
        width: u32,
        height: u32,
    ) {
        let old_view = self.get_view();

        // Convert the focus point to fractal coordinates before zoom
        let focus = self
            .viewport
            .screen_to_world(screen_x, screen_y, width, height);

        let mut view = old_view.clone();
        view.zoom *= factor;

        // Adjust center so the focus point stays under the cursor
        // Before zoom: focus_world = center + offset/zoom_old
        // After zoom: we want focus_world at the same screen position
        // new_center = focus_world - offset/zoom_new = focus_world - (focus_world - old_center)*(zoom_old/zoom_new)
        let ratio = 1.0 / factor;
        view.center_x = focus.re - (focus.re - old_view.center_x) * ratio;
        view.center_y = focus.im - (focus.im - old_view.center_y) * ratio;

        if self.render.adaptive_iterations {
            let new_iter = self.calculate_adaptive_iterations(view.zoom);
            view.max_iterations = new_iter;
            self.controls.max_iterations = new_iter;
            self.controls.pending_max_iterations = new_iter;
        }

        self.set_view(view.clone());
        self.execute_view_command(&old_view, &view);
        self.invalidate_cache();
    }

    fn zoom_view(&mut self, factor: f64) {
        let old_view = self.get_view();
        let mut view = old_view.clone();
        view.zoom *= factor;

        if self.render.adaptive_iterations {
            let new_iter = self.calculate_adaptive_iterations(view.zoom);
            view.max_iterations = new_iter;
            self.controls.max_iterations = new_iter;
            self.controls.pending_max_iterations = new_iter;
        }

        self.set_view(view.clone());

        // Execute command for history
        self.execute_view_command(&old_view, &view);

        self.invalidate_cache();
    }

    fn pan_view(&mut self, dx: f64, dy: f64) {
        let old_view = self.get_view();
        let mut view = old_view.clone();
        let pan_amount = PAN_AMOUNT_BASE / view.zoom;
        view.center_x += dx * pan_amount;
        view.center_y += dy * pan_amount;
        self.set_view(view.clone());

        // Execute command for history
        self.execute_view_command(&old_view, &view);

        // Try to optimize pan by shifting existing pixels
        if let Some(ref mut cached) = self.render.cached_image {
            let regions = self
                .render
                .engine
                .calculate_pan_regions(cached, dx, dy, view.zoom);

            if !regions.is_empty() {
                self.render.partial_render_regions = regions;
                self.render.current_region_index = 0;
                self.render.needs_render = true;
                return;
            }
        }

        self.render.needs_render = true;
    }

    fn undo(&mut self) {
        let mut state = self.to_app_state();
        if let Some(description) = self.get_command_history().undo(&mut state) {
            self.apply_app_state(&state);
            self.fractal = self.create_fractal(state.fractal_type);
            self.invalidate_cache();
            self.set_status(format!("Undo: {}", description));
        }
    }

    fn redo(&mut self) {
        let mut state = self.to_app_state();
        if let Some(description) = self.get_command_history().redo(&mut state) {
            self.apply_app_state(&state);
            self.fractal = self.create_fractal(state.fractal_type);
            self.invalidate_cache();
            self.set_status(format!("Redo: {}", description));
        }
    }

    fn add_bookmark(&mut self, name: String) {
        let view = self.get_view();
        let bookmark = Bookmark {
            name,
            fractal_type: self.controls.fractal_type,
            center_x: view.center_x,
            center_y: view.center_y,
            zoom: view.zoom,
            max_iterations: view.max_iterations,
            palette_type: view.palette_type,
            color_processor_type: view.color_processor_type,
            fractal_params: view.fractal_params.clone(),
        };
        self.bookmarks.push(bookmark);
        self.set_status("Bookmark saved".to_string());
    }

    fn delete_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
            self.set_status("Bookmark deleted".to_string());
        }
    }

    fn load_bookmark(&mut self, index: usize) {
        if let Some(bookmark) = self.bookmarks.get(index).cloned() {
            self.controls.fractal_type = bookmark.fractal_type;
            self.fractal = self.create_fractal(bookmark.fractal_type);

            // Restore fractal parameters
            for (name, value) in &bookmark.fractal_params {
                self.fractal.set_parameter(name, *value);
            }

            let view = FractalViewState {
                center_x: bookmark.center_x,
                center_y: bookmark.center_y,
                zoom: bookmark.zoom,
                max_iterations: bookmark.max_iterations,
                fractal_params: bookmark.fractal_params.clone(),
                palette_type: bookmark.palette_type,
                color_processor_type: bookmark.color_processor_type,
            };
            self.set_view(view);

            self.controls.max_iterations = bookmark.max_iterations;
            self.controls.pending_max_iterations = bookmark.max_iterations;
            self.controls.palette_type = bookmark.palette_type;
            self.controls.color_processor_type = bookmark.color_processor_type;
            self.controls.pending_fractal_params = bookmark.fractal_params.clone();

            self.invalidate_cache();
            self.set_status(format!("Loaded: {}", bookmark.name));
        }
    }

    fn set_status(&mut self, message: String) {
        self.interaction.status_message = Some((message, Instant::now()));
    }

    fn check_status_timeout(&mut self) {
        if let Some((_, timestamp)) = self.interaction.status_message {
            if timestamp.elapsed().as_secs_f64() > STATUS_TIMEOUT_SECS {
                self.interaction.status_message = None;
            }
        }
    }

    fn update_mouse_position(&mut self, pos: egui::Pos2, rect: &egui::Rect) {
        let width = rect.width() as u32;
        let height = rect.height() as u32;

        let x = (pos.x - rect.min.x) as u32;
        let y = (pos.y - rect.min.y) as u32;

        if x < width && y < height {
            let world = self.viewport.screen_to_world(x, y, width, height);
            self.interaction.mouse_fractal_pos = Some((world.re, world.im));
        } else {
            self.interaction.mouse_fractal_pos = None;
        }
    }

    fn render_minimap(&mut self, ctx: &egui::Context) {
        if !self.minimap_enabled {
            return;
        }

        if !self.minimap_dirty && self.cached_minimap_texture.is_some() {
            // Just update the view rectangle overlay - reuse the cached fractal rendering
            // We re-render only the view rectangle (cheap) on top of the cached fractal minimap
            return;
        }

        let minimap_size = MINIMAP_SIZE;
        let mut pixels = vec![egui::Color32::BLACK; minimap_size * minimap_size];

        let max_iter = MINIMAP_MAX_ITER;

        let minimap_viewport = Viewport::from_view(
            self.controls.fractal_type.default_center().0,
            self.controls.fractal_type.default_center().1,
            1.0,
            minimap_size as u32,
            minimap_size as u32,
        );

        for y in 0..minimap_size {
            for x in 0..minimap_size {
                let world = minimap_viewport.screen_to_world(
                    x as u32,
                    y as u32,
                    minimap_size as u32,
                    minimap_size as u32,
                );
                let iterations = self.fractal.compute(world.re, world.im, max_iter);
                let color = if iterations >= max_iter {
                    egui::Color32::BLACK
                } else {
                    let t = iterations as f32 / max_iter as f32;
                    palette::get_color(self.controls.palette_type, t, 0.0)
                };
                pixels[y * minimap_size + x] = color;
            }
        }

        // Draw view rectangle
        let default_center = self.controls.fractal_type.default_center();
        let (view_center_x, view_center_y) = self.viewport.center();
        let view_zoom = self.viewport.zoom();
        let view_width = 4.0 / view_zoom;
        let view_height = view_width;

        let map_range = MINIMAP_MAP_RANGE;
        let rel_x = (view_center_x - default_center.0 + map_range / 2.0) / map_range;
        let rel_y = (view_center_y - default_center.1 + map_range / 2.0) / map_range;

        let rect_x = (rel_x * minimap_size as f64) as i32;
        let rect_y = (rel_y * minimap_size as f64) as i32;
        let rect_w = ((view_width / map_range) * minimap_size as f64) as i32;
        let rect_h = ((view_height / map_range) * minimap_size as f64) as i32;

        for dy in -rect_h / 2..=rect_h / 2 {
            for dx in -rect_w / 2..=rect_w / 2 {
                if dx == -rect_w / 2 || dx == rect_w / 2 || dy == -rect_h / 2 || dy == rect_h / 2 {
                    let px = rect_x + dx;
                    let py = rect_y + dy;
                    if px >= 0 && px < minimap_size as i32 && py >= 0 && py < minimap_size as i32 {
                        pixels[py as usize * minimap_size + px as usize] = egui::Color32::YELLOW;
                    }
                }
            }
        }

        let image = egui::ColorImage {
            size: [minimap_size, minimap_size],
            pixels,
        };

        self.cached_minimap_texture =
            Some(ctx.load_texture("minimap", image, egui::TextureOptions::default()));
        self.minimap_dirty = false;
    }
}

impl eframe::App for FractalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_status_timeout();

        // Track actual window size for saving on exit
        ctx.input(|i| {
            if let Some(size) = i.viewport().inner_rect {
                self.actual_window_width = size.width();
                self.actual_window_height = size.height();
            }
        });

        // Handle keyboard input (disable when bookmark dialog is open)
        if !self.show_bookmark_dialog {
            ctx.input(|i| {
                // Zoom controls: +/- keys
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    self.zoom_view(ZOOM_KEYBOARD_FACTOR);
                }
                if i.key_pressed(egui::Key::Minus) {
                    self.zoom_view(1.0 / ZOOM_KEYBOARD_FACTOR);
                }

                // Pan controls: arrow keys
                if i.key_pressed(egui::Key::ArrowLeft) {
                    self.pan_view(-1.0, 0.0);
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    self.pan_view(1.0, 0.0);
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    self.pan_view(0.0, 1.0);
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.pan_view(0.0, -1.0);
                }

                // Reset view: R key
                if i.key_pressed(egui::Key::R) && !i.modifiers.shift {
                    self.reset_view();
                    self.invalidate_cache();
                }

                // Undo/Redo
                if i.key_pressed(egui::Key::Z) && i.modifiers.ctrl {
                    self.undo();
                }
                if i.key_pressed(egui::Key::Y) && i.modifiers.ctrl {
                    self.redo();
                }

                // Save: S key
                if i.key_pressed(egui::Key::S) {
                    match self.save_image(1) {
                        Ok(path) => {
                            self.set_status(format!("Saved: {}", path.display()));
                        }
                        Err(e) => {
                            self.set_status(format!("Error: {}", e));
                        }
                    }
                }
            });
        }

        egui::SidePanel::left("controls")
            .default_width(CONTROL_PANEL_WIDTH)
            .show(ctx, |ui| {
                let prev_fractal = self.controls.fractal_type;
                let mut changed = false;
                let render_status = RenderStatus::new(
                    self.render.is_rendering || self.render.needs_render,
                    self.render.render_progress,
                    self.render.last_render_time,
                    rayon::current_num_threads(),
                );
                self.controls
                    .ui(ui, &mut self.fractal, &mut changed, &render_status);

                if prev_fractal != self.controls.fractal_type {
                    self.fractal = self.create_fractal(self.controls.fractal_type);
                    if let Some(view) = self.views.get(&self.controls.fractal_type) {
                        let view = view.clone();
                        self.controls.max_iterations = view.max_iterations;
                        self.controls.pending_max_iterations = view.max_iterations;
                        self.controls.pending_fractal_params = view.fractal_params.clone();
                        self.controls.palette_type = view.palette_type;
                        self.controls.color_processor_type = view.color_processor_type;
                        self.controls.pending_palette_offset = self.controls.palette_offset;
                        for (name, value) in &view.fractal_params {
                            self.fractal.set_parameter(name, *value);
                        }
                        // Update viewport to match the restored view
                        self.set_view(view);
                    }
                    self.invalidate_cache();
                }

                if changed {
                    if let Some(view) = self.views.get_mut(&self.controls.fractal_type) {
                        view.max_iterations = self.controls.max_iterations;
                        view.fractal_params = self.controls.pending_fractal_params.clone();
                        view.palette_type = self.controls.palette_type;
                        view.color_processor_type = self.controls.color_processor_type;
                    }
                    self.invalidate_cache();
                }

                ui.separator();

                // View controls
                ui.horizontal(|ui| {
                    if ui.button("Reset View (R)").clicked() {
                        self.reset_view();
                        self.invalidate_cache();
                    }
                    if ui
                        .button("Reset All")
                        .on_hover_text("Reset view, palette, and parameters")
                        .clicked()
                    {
                        self.reset_settings();
                    }
                });

                let can_undo = self.get_command_history().can_undo();
                let can_redo = self.get_command_history().can_redo();
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo (^Z)"))
                        .clicked()
                    {
                        self.undo();
                    }
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Redo (^Y)"))
                        .clicked()
                    {
                        self.redo();
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save (S)").clicked() {
                        match self.save_image(self.export_scale) {
                            Ok(path) => self.set_status(format!(
                                "Saved {}x: {}",
                                self.export_scale,
                                path.display()
                            )),
                            Err(e) => self.set_status(format!("Error: {}", e)),
                        }
                    }
                    ui.radio_value(&mut self.export_scale, 1, "1x");
                    ui.radio_value(&mut self.export_scale, 2, "2x");
                    ui.radio_value(&mut self.export_scale, 4, "4x");
                });

                ui.separator();

                // Settings toggles
                let prev_supersampling = self.render.supersampling_enabled;
                ui.checkbox(&mut self.render.supersampling_enabled, "Supersampling (2x)");
                if self.render.supersampling_enabled != prev_supersampling {
                    self.invalidate_cache();
                }

                let prev_adaptive = self.render.adaptive_iterations;
                ui.checkbox(&mut self.render.adaptive_iterations, "Adaptive Iterations");
                if self.render.adaptive_iterations != prev_adaptive {
                    self.invalidate_cache();
                }
                if self.render.adaptive_iterations {
                    ui.label(format!(
                        "Current: {}",
                        self.calculate_adaptive_iterations(self.get_view().zoom)
                    ));
                }

                let prev_minimap = self.minimap_enabled;
                ui.checkbox(&mut self.minimap_enabled, "Show Minimap");
                if self.minimap_enabled != prev_minimap {
                    self.invalidate_cache();
                }

                // Bookmark dialog
                if self.show_bookmark_dialog {
                    ui.separator();
                    ui.label("Bookmark Name:");
                    ui.text_edit_singleline(&mut self.bookmark_name_input);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() && !self.bookmark_name_input.is_empty() {
                            self.add_bookmark(self.bookmark_name_input.clone());
                            self.show_bookmark_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_bookmark_dialog = false;
                        }
                    });
                }

                // Bookmarks list
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Bookmarks:");
                    if ui.button("Add").clicked() {
                        self.show_bookmark_dialog = true;
                        self.bookmark_name_input.clear();
                    }
                });

                // Show bookmark status message if present
                if let Some((msg, _)) = &self.interaction.status_message {
                    ui.label(egui::RichText::new(msg).color(egui::Color32::YELLOW));
                }

                if !self.bookmarks.is_empty() {
                    let mut load_index = None;
                    let mut delete_index = None;
                    egui::ScrollArea::vertical()
                        .max_height(BOOKMARK_SCROLL_HEIGHT)
                        .show(ui, |ui| {
                            for i in 0..self.bookmarks.len() {
                                ui.horizontal(|ui| {
                                    if ui.button(&self.bookmarks[i].name).clicked() {
                                        load_index = Some(i);
                                    }
                                    if ui.button("×").clicked() {
                                        delete_index = Some(i);
                                    }
                                });
                            }
                        });
                    if let Some(i) = load_index {
                        self.load_bookmark(i);
                    }
                    if let Some(i) = delete_index {
                        self.delete_bookmark(i);
                    }
                }

                if self.interaction.drag_start.is_some() {
                    ui.separator();
                    ui.label("Release to apply zoom");
                }

                ui.separator();
                let view = self.get_view();
                ui.label(format!(
                    "Center: ({:.6}, {:.6}) x {:.2e}",
                    view.center_x, view.center_y, view.zoom
                ));

                // Mouse coordinates display
                if let Some((fx, fy)) = self.interaction.mouse_fractal_pos {
                    ui.separator();
                    ui.label(format!("Cursor: ({:.6}, {:.6})", fx, fy));
                }

                ui.separator();
                ui.label("Mouse:");
                ui.label("Click + Drag: Select zoom region");
                ui.label("Wheel: Zoom in/out at cursor");

                ui.separator();
                ui.label("Keyboard:");
                ui.label("+/- : Zoom in/out");
                ui.label("Arrows : Pan");
                ui.label("R : Reset view");
                ui.label("Shift+R : Reset all");
                ui.label("Ctrl+Z : Undo");
                ui.label("Ctrl+Y : Redo");
                ui.label("S : Save image");

                ui.separator();
                if ui.button("About").clicked() {
                    self.show_about_dialog = true;
                }
            });

        // About dialog
        if self.show_about_dialog {
            // Load about image once and cache it
            if self.cached_about_texture.is_none() {
                let image_path = ABOUT_IMAGE_PATH;
                if let Ok(image_data) = std::fs::read(image_path) {
                    if let Ok(image) = image::load_from_memory(&image_data) {
                        let rgba = image.to_rgba8();
                        let size = [image.width() as _, image.height() as _];
                        let pixels: Vec<egui::Color32> = rgba
                            .pixels()
                            .map(|p| egui::Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
                            .collect();
                        let color_image = egui::ColorImage { size, pixels };
                        self.cached_about_texture = Some(ctx.load_texture(
                            "about_image",
                            color_image,
                            egui::TextureOptions::default(),
                        ));
                    }
                }
            }

            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(ref texture) = self.cached_about_texture {
                        ui.image((
                            texture.id(),
                            egui::vec2(ABOUT_IMAGE_DISPLAY_WIDTH, ABOUT_IMAGE_DISPLAY_HEIGHT),
                        ));
                    } else {
                        ui.label("Image not found");
                    }

                    ui.separator();
                    ui.label("Fractal Oxide\nCopyright © 2026 ultrametrics");

                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let width = rect.width() as u32;
            let height = rect.height() as u32;

            if width == 0 || height == 0 {
                return;
            }

            // Update viewport dimensions if changed
            if width != self.render.cached_width || height != self.render.cached_height {
                self.update_viewport_dimensions(width, height);
            }

            let response =
                ui.interact(rect, egui::Id::new("canvas"), egui::Sense::click_and_drag());

            let mut pointer_pos = None;
            ctx.input(|i| {
                pointer_pos = i.pointer.interact_pos();
            });

            // Update mouse position for coordinate display
            if let Some(pos) = pointer_pos {
                self.update_mouse_position(pos, &rect);
            } else {
                self.interaction.mouse_fractal_pos = None;
            }

            // Scroll-wheel zoom at cursor position
            if response.hovered() {
                let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll_delta.abs() > SCROLL_DEADZONE {
                    if let Some(pos) = pointer_pos {
                        let sx = (pos.x - rect.min.x) as u32;
                        let sy = (pos.y - rect.min.y) as u32;
                        if sx < width && sy < height {
                            let factor = if scroll_delta > 0.0 {
                                1.0 + scroll_delta as f64 * SCROLL_ZOOM_SENSITIVITY
                            } else {
                                1.0 / (1.0 + (-scroll_delta) as f64 * SCROLL_ZOOM_SENSITIVITY)
                            };
                            self.zoom_at_point(factor, sx, sy, width, height);
                        }
                    }
                }
            }

            if response.drag_started() {
                self.interaction.drag_start = pointer_pos;
                self.interaction.drag_current = pointer_pos;
                self.interaction.zoom_preview = None;
            }

            if response.dragged() {
                if let Some(pos) = pointer_pos {
                    self.interaction.drag_current = Some(pos);
                }
                ctx.request_repaint();
            }

            if response.drag_stopped() {
                if let (Some(start), Some(end)) =
                    (self.interaction.drag_start, self.interaction.drag_current)
                {
                    let dx = (end.x - start.x).abs();
                    let dy = (end.y - start.y).abs();

                    if dx > DRAG_THRESHOLD_PX || dy > DRAG_THRESHOLD_PX {
                        let min_x = start.x.min(end.x) - rect.min.x;
                        let max_x = start.x.max(end.x) - rect.min.x;
                        let min_y = start.y.min(end.y) - rect.min.y;
                        let max_y = start.y.max(end.y) - rect.min.y;

                        self.render.prev_image = self.render.cached_image.clone();

                        self.interaction.zoom_preview = Some(ZoomPreview {
                            sel_min: egui::pos2(min_x, min_y),
                            sel_max: egui::pos2(max_x, max_y),
                        });

                        let view = self.get_view();

                        let tl = self.viewport.screen_to_world(
                            min_x as u32,
                            min_y as u32,
                            width,
                            height,
                        );
                        let br = self.viewport.screen_to_world(
                            max_x as u32,
                            max_y as u32,
                            width,
                            height,
                        );

                        let new_center_x = (tl.re + br.re) / 2.0;
                        let new_center_y = (tl.im + br.im) / 2.0;

                        let sel_height_px = max_y - min_y;
                        let new_zoom = view.zoom * (height as f64 / sel_height_px as f64);

                        // Calculate adaptive iterations if enabled
                        let new_max_iter = if self.render.adaptive_iterations {
                            self.calculate_adaptive_iterations(new_zoom)
                        } else {
                            self.controls.max_iterations
                        };

                        let old_view = self.get_view();

                        let new_view = FractalViewState {
                            center_x: new_center_x,
                            center_y: new_center_y,
                            zoom: new_zoom,
                            max_iterations: new_max_iter,
                            fractal_params: view.fractal_params.clone(),
                            palette_type: self.controls.palette_type,
                            color_processor_type: self.controls.color_processor_type,
                        };
                        self.set_view(new_view.clone());

                        // Execute command for history
                        self.execute_view_command(&old_view, &new_view);

                        // Update controls to reflect new iteration count
                        if self.render.adaptive_iterations {
                            self.controls.max_iterations = new_max_iter;
                            self.controls.pending_max_iterations = new_max_iter;
                        }

                        self.render.render_delay = RENDER_DELAY_FRAMES;
                    }
                }

                self.interaction.drag_start = None;
                self.interaction.drag_current = None;
                ctx.request_repaint();
            }

            // Initial render check (pause when bookmark dialog is open)
            if !self.show_bookmark_dialog {
                if self.render.cached_width == 0 || self.render.cached_height == 0 {
                    self.invalidate_cache();
                } else if self.render.render_delay > 0 {
                    self.render.render_delay -= 1;
                    if self.render.render_delay == 0 {
                        self.invalidate_cache();
                    }
                }
            }

            // Main fractal display - update texture only when image changes
            if self.render.texture_dirty {
                if let Some(ref image) = self.render.cached_image {
                    self.render.cached_texture = Some(ctx.load_texture(
                        "fractal",
                        image.clone(),
                        egui::TextureOptions::default(),
                    ));
                    self.render.texture_dirty = false;
                }
            }
            if let Some(ref texture) = self.render.cached_texture {
                ui.put(
                    egui::Rect::from_min_size(rect.min, rect.size()),
                    egui::Image::new((texture.id(), rect.size())).uv(egui::Rect::from_min_max(
                        egui::pos2(0.0, 0.0),
                        egui::pos2(1.0, 1.0),
                    )),
                );
            }

            // Draw minimap if enabled (must be before getting painter)
            self.render_minimap(ctx);
            let minimap_rect = if self.minimap_enabled {
                if let Some(ref minimap_texture) = self.cached_minimap_texture {
                    let minimap_size = MINIMAP_SIZE as f32;
                    let minimap_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.max.x - minimap_size - 10.0, rect.min.y + 10.0),
                        egui::vec2(minimap_size, minimap_size),
                    );
                    ui.put(
                        minimap_rect,
                        egui::Image::new((minimap_texture.id(), minimap_rect.size())),
                    );
                    Some(minimap_rect)
                } else {
                    None
                }
            } else {
                None
            };

            let painter = ui.painter();

            // Draw zoom preview if available
            if let Some(ref preview) = self.interaction.zoom_preview {
                if let Some(ref image) = self.render.prev_image {
                    let texture = ctx.load_texture(
                        "fractal_preview",
                        image.clone(),
                        egui::TextureOptions::default(),
                    );
                    let uv_min = egui::pos2(
                        (preview.sel_min.x / rect.width()).clamp(0.0, 1.0),
                        (preview.sel_min.y / rect.height()).clamp(0.0, 1.0),
                    );
                    let uv_max = egui::pos2(
                        (preview.sel_max.x / rect.width()).clamp(0.0, 1.0),
                        (preview.sel_max.y / rect.height()).clamp(0.0, 1.0),
                    );
                    painter.image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(uv_min, uv_max),
                        egui::Color32::WHITE,
                    );
                }
            }

            // Draw selection rectangle outline
            if self.interaction.zoom_preview.is_none() {
                if let (Some(start), Some(end)) =
                    (self.interaction.drag_start, self.interaction.drag_current)
                {
                    let sel_rect = egui::Rect::from_two_pos(start, end);
                    painter.rect_stroke(sel_rect, 1.0, egui::Stroke::new(2.0, egui::Color32::BLUE));
                }
            }

            // Draw border around minimap
            if let Some(minimap_rect) = minimap_rect {
                painter.rect_stroke(
                    minimap_rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
            }

            // Rendering logic using the new RenderEngine
            if self.render.is_rendering {
                if let Some(ref config) = self.render.config.clone() {
                    if !self.render.partial_render_regions.is_empty() {
                        // Partial rendering for pan optimization
                        if self.render.current_region_index
                            < self.render.partial_render_regions.len()
                        {
                            let region = &self.render.partial_render_regions
                                [self.render.current_region_index];
                            let chunk_size = ((region.height as f64 / 10.0).ceil() as u32).max(1);

                            if let Some(chunk_result) = self.render.engine.render_region(
                                region,
                                self.fractal.as_ref(),
                                &self.get_view(),
                                config,
                                self.render.render_chunk_start,
                                chunk_size,
                            ) {
                                // Update cached image with rendered pixels
                                if let Some(ref mut cached) = self.render.cached_image {
                                    for dy in 0..chunk_result.height {
                                        let y = chunk_result.y + dy;
                                        for dx in 0..chunk_result.width {
                                            let x = chunk_result.x + dx;
                                            let src_idx = (dy * chunk_result.width + dx) as usize;
                                            let dst_idx = (y * cached.width() as u32 + x) as usize;
                                            if dst_idx < cached.pixels.len() {
                                                cached.pixels[dst_idx] =
                                                    chunk_result.pixels[src_idx];
                                            }
                                        }
                                    }
                                }

                                self.render.render_chunk_start += chunk_result.height;
                                if self.render.render_chunk_start >= region.height {
                                    self.render.current_region_index += 1;
                                    self.render.render_chunk_start = 0;
                                }

                                self.render.texture_dirty = true;
                                self.render.render_progress = (self.render.current_region_index
                                    as f32
                                    + self.render.render_chunk_start as f32 / region.height as f32)
                                    / self.render.partial_render_regions.len() as f32;
                                ctx.request_repaint();
                            } else {
                                // Region complete
                                self.render.current_region_index += 1;
                                self.render.render_chunk_start = 0;
                                ctx.request_repaint();
                            }
                        } else {
                            // All regions complete
                            self.render.is_rendering = false;
                            self.render.render_progress = 0.0;
                            self.render.partial_render_regions.clear();
                            self.render.current_region_index = 0;
                            self.render.render_chunk_start = 0;
                            self.render.config = None;

                            if let Some(start_time) = self.render.render_start_time.take() {
                                self.render.last_render_time =
                                    Some(start_time.elapsed().as_secs_f64());
                            }
                            ctx.request_repaint();
                        }
                    } else {
                        // Full canvas rendering
                        let (_render_width, render_height) = config.render_dimensions();
                        let chunk_size = ((render_height as f64 / 60.0).ceil() as u32).max(1);

                        let has_more = self.render.engine.render_full_chunk(
                            self.fractal.as_ref(),
                            &self.get_view(),
                            config,
                            self.render.render_chunk_start,
                            chunk_size,
                        );

                        if has_more {
                            self.render.render_chunk_start +=
                                chunk_size.min(render_height - self.render.render_chunk_start);
                            self.render.render_progress =
                                self.render.render_chunk_start as f32 / render_height as f32;
                            ctx.request_repaint();
                        } else {
                            // Rendering complete
                            if let Some(pixels) = self.render.engine.finalize(config) {
                                self.render.cached_image = Some(egui::ColorImage {
                                    size: [config.width as _, config.height as _],
                                    pixels,
                                });
                                self.render.texture_dirty = true;
                            }

                            self.render.cached_width = config.width;
                            self.render.cached_height = config.height;
                            self.render.needs_render = false;
                            self.render.is_rendering = false;
                            self.render.render_progress = 0.0;
                            self.interaction.zoom_preview = None;
                            self.render.render_chunk_start = 0;
                            self.render.config = None;

                            if let Some(start_time) = self.render.render_start_time.take() {
                                self.render.last_render_time =
                                    Some(start_time.elapsed().as_secs_f64());
                            }
                            ctx.request_repaint();
                        }
                    }
                }
            }

            // Start new render if needed
            if self.render.needs_render && !self.render.is_rendering && !self.show_bookmark_dialog {
                let view = self.get_view();
                let max_iter = if self.render.adaptive_iterations {
                    self.calculate_adaptive_iterations(view.zoom)
                } else {
                    self.controls.max_iterations
                };

                let config = RenderConfig {
                    width,
                    height,
                    supersampling: self.render.supersampling_enabled,
                    max_iterations: max_iter,
                    palette_type: self.controls.palette_type,
                    palette_offset: self.controls.palette_offset,
                    color_pipeline: color_pipeline::ColorPipeline::from_type(
                        view.color_processor_type,
                    ),
                };

                self.render.engine.start_render(&config);
                self.render.config = Some(config);
                self.render.is_rendering = true;
                self.render.render_start_time = Some(Instant::now());
                self.render.render_progress = 0.0;
                self.render.render_chunk_start = 0;
                self.render.current_region_index = 0;
                self.render.needs_render = false;
                ctx.request_repaint();
            }
        });
    }

    fn on_exit(&mut self, _ctx: Option<&eframe::glow::Context>) {
        // Save window size on exit
        let config = AppConfig {
            window_width: self.actual_window_width,
            window_height: self.actual_window_height,
            default_iterations: self.controls.max_iterations,
            default_fractal: self.controls.fractal_type,
            default_palette: self.controls.palette_type,
            supersampling_enabled: self.render.supersampling_enabled,
            adaptive_iterations: self.render.adaptive_iterations,
            bookmarks: self.bookmarks.clone(),
        };
        if let Err(e) = config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }
}

fn main() -> eframe::Result {
    eprintln!("STARTING Fractal Oxide...");

    let config = AppConfig::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.window_width, config.window_height])
            .with_title("Fractal Oxide"),
        ..Default::default()
    };

    eframe::run_native(
        "Fractal Oxide",
        options,
        Box::new(|_cc| Ok(Box::new(FractalApp::new(&config)))),
    )
}
