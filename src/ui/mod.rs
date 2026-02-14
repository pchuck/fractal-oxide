use eframe::egui;
use std::collections::HashMap;

use crate::color_pipeline::ColorProcessorType;
use crate::fractal::{Fractal, FractalType};
use crate::palette::PaletteType;

/// Render status information for display in UI
pub struct RenderStatus {
    pub is_rendering: bool,
    pub render_progress: f32,
    pub last_render_time: Option<f64>, // in seconds
    pub thread_count: usize,
}

impl RenderStatus {
    pub fn new(
        is_rendering: bool,
        render_progress: f32,
        last_render_time: Option<f64>,
        thread_count: usize,
    ) -> Self {
        Self {
            is_rendering,
            render_progress,
            last_render_time,
            thread_count,
        }
    }
}

pub struct FractalControls {
    pub fractal_type: FractalType,
    pub palette_type: PaletteType,
    pub color_processor_type: ColorProcessorType,
    pub max_iterations: u32,
    pub palette_offset: f32,
    pub pending_max_iterations: u32,
    pub pending_palette_offset: f32,
    pub pending_fractal_params: HashMap<String, f64>,
}

impl Default for FractalControls {
    fn default() -> Self {
        FractalControls {
            fractal_type: FractalType::Mandelbrot,
            palette_type: PaletteType::Classic,
            color_processor_type: ColorProcessorType::Palette,
            max_iterations: 200,
            palette_offset: 0.0,
            pending_max_iterations: 200,
            pending_palette_offset: 0.0,
            pending_fractal_params: HashMap::new(),
        }
    }
}

impl FractalControls {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        fractal: &mut Box<dyn Fractal>,
        changed: &mut bool,
        render_status: &RenderStatus,
    ) {
        ui.heading("Fractal Oxide");
        ui.separator();

        // Fractal Type and Render Status side by side
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Fractal Type:");
                egui::ComboBox::from_id_salt("fractal_type")
                    .selected_text(match self.fractal_type {
                        FractalType::Mandelbrot => "Mandelbrot",
                        FractalType::Julia => "Julia",
                        FractalType::BurningShip => "Burning Ship",
                        FractalType::Tricorn => "Tricorn",
                        FractalType::Celtic => "Celtic",
                        FractalType::Newton => "Newton",
                        FractalType::Biomorph => "Biomorph",
                        FractalType::Phoenix => "Phoenix",
                        FractalType::Multibrot => "Multibrot",
                        FractalType::Spider => "Spider",
                        FractalType::OrbitTrap => "Orbit Trap",
                        FractalType::PickoverStalk => "Pickover Stalk",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::Mandelbrot,
                            "Mandelbrot",
                        );
                        ui.selectable_value(&mut self.fractal_type, FractalType::Julia, "Julia");
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::BurningShip,
                            "Burning Ship",
                        );
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::Tricorn,
                            "Tricorn",
                        );
                        ui.selectable_value(&mut self.fractal_type, FractalType::Celtic, "Celtic");
                        ui.selectable_value(&mut self.fractal_type, FractalType::Newton, "Newton");
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::Biomorph,
                            "Biomorph",
                        );
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::Phoenix,
                            "Phoenix",
                        );
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::Multibrot,
                            "Multibrot",
                        );
                        ui.selectable_value(&mut self.fractal_type, FractalType::Spider, "Spider");
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::OrbitTrap,
                            "Orbit Trap",
                        );
                        ui.selectable_value(
                            &mut self.fractal_type,
                            FractalType::PickoverStalk,
                            "Pickover Stalk",
                        );
                    });
            });

            ui.add(egui::Separator::default().vertical());

            ui.vertical(|ui| {
                // Show render status
                if render_status.is_rendering {
                    ui.label(format!("Parallel: {} threads", render_status.thread_count));
                    ui.label("Rendering...");
                    ui.add(
                        egui::ProgressBar::new(render_status.render_progress).desired_width(120.0),
                    );
                } else if let Some(render_time) = render_status.last_render_time {
                    ui.label(format!("Parallel: {} threads", render_status.thread_count));
                    if render_time < 1.0 {
                        ui.label(format!("Last render: {:.0}ms", render_time * 1000.0));
                    } else {
                        ui.label(format!("Last render: {:.2}s", render_time));
                    }
                } else {
                    ui.label(format!("Parallel: {} threads", render_status.thread_count));
                    ui.label("Ready");
                }
            });
        });

        ui.separator();

        // Track changes before modifying
        let prev_palette = self.palette_type;
        let prev_processor = self.color_processor_type;

        // Color Palette and Processor side by side
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Color Palette:");
                egui::ComboBox::from_id_salt("palette_type")
                    .selected_text(match self.palette_type {
                        PaletteType::Classic => "Classic",
                        PaletteType::Fire => "Fire",
                        PaletteType::Ice => "Ice",
                        PaletteType::Grayscale => "Grayscale",
                        PaletteType::Psychedelic => "Psychedelic",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.palette_type,
                            PaletteType::Classic,
                            "Classic",
                        );
                        ui.selectable_value(&mut self.palette_type, PaletteType::Fire, "Fire");
                        ui.selectable_value(&mut self.palette_type, PaletteType::Ice, "Ice");
                        ui.selectable_value(
                            &mut self.palette_type,
                            PaletteType::Grayscale,
                            "Grayscale",
                        );
                        ui.selectable_value(
                            &mut self.palette_type,
                            PaletteType::Psychedelic,
                            "Psychedelic",
                        );
                    });
            });

            ui.add(egui::Separator::default().vertical());

            ui.vertical(|ui| {
                ui.label("Color Processor:");
                egui::ComboBox::from_id_salt("color_processor")
                    .selected_text(self.color_processor_type.display_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.color_processor_type,
                            ColorProcessorType::Palette,
                            ColorProcessorType::Palette.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.color_processor_type,
                            ColorProcessorType::Smooth,
                            ColorProcessorType::Smooth.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.color_processor_type,
                            ColorProcessorType::OrbitTrapReal,
                            ColorProcessorType::OrbitTrapReal.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.color_processor_type,
                            ColorProcessorType::OrbitTrapImag,
                            ColorProcessorType::OrbitTrapImag.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.color_processor_type,
                            ColorProcessorType::OrbitTrapOrigin,
                            ColorProcessorType::OrbitTrapOrigin.display_name(),
                        );
                    });
            });
        });

        let mut palette_changed = prev_palette != self.palette_type;
        if prev_processor != self.color_processor_type {
            *changed = true;
        }
        if self.palette_type == PaletteType::Psychedelic {
            ui.label("Color Offset:");
            let response = ui
                .add(egui::Slider::new(&mut self.pending_palette_offset, 0.0..=1.0).text("offset"));
            if response.drag_stopped() {
                self.palette_offset = self.pending_palette_offset;
                palette_changed = true;
            }
        } else {
            self.pending_palette_offset = self.palette_offset;
        }

        ui.separator();
        ui.label("Iterations:");
        let response =
            ui.add(egui::Slider::new(&mut self.pending_max_iterations, 16..=2000).text("max_iter"));
        if response.drag_stopped() {
            self.max_iterations = self.pending_max_iterations;
            *changed = true;
        }

        ui.separator();
        ui.label("Fractal Parameters:");

        for param in fractal.parameters() {
            let mut value = self
                .pending_fractal_params
                .get(&param.name)
                .copied()
                .unwrap_or(param.value);
            let response =
                ui.add(egui::Slider::new(&mut value, param.min..=param.max).text(&param.name));
            self.pending_fractal_params
                .insert(param.name.clone(), value);
            if response.drag_stopped() {
                fractal.set_parameter(&param.name, value);
                *changed = true;
            }
        }

        if palette_changed {
            *changed = true;
        }
    }
}
