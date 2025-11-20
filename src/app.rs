// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Main application state and egui App implementation.
//!
//! This module contains the main application structure that implements
//! the egui::App trait, managing the overall application state and
//! coordinating between different UI components and the data model.

use crate::models::project::ProjectData;
use crate::ui::{canvas, properties, timeline, toolbar};

/// Current drawing tool selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Polygon,
    Line,
}

/// Main application state.
pub struct RoidsApp {
    /// Currently selected drawing tool
    current_tool: Tool,

    /// Current project data (if a file is loaded)
    project: Option<ProjectData>,

    /// Index of currently selected annotation
    selected_annotation: Option<usize>,

    /// Whether a video file is loaded (shows timeline if true)
    is_video: bool,

    /// Loaded image texture for display
    image_texture: Option<egui::TextureHandle>,

    /// Image dimensions (width, height)
    image_size: Option<(u32, u32)>,
}

impl Default for RoidsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RoidsApp {
    /// Create a new ROIDS application instance.
    pub fn new() -> Self {
        Self {
            current_tool: Tool::Select,
            project: None,
            selected_annotation: None,
            is_video: false,
            image_texture: None,
            image_size: None,
        }
    }

    /// Load an image file and create a texture for display.
    pub fn load_image_file(&mut self, path: std::path::PathBuf, ctx: &egui::Context) {
        match crate::io::media::load_image(&path) {
            Ok(loaded_img) => {
                // Create egui texture from the loaded image
                let size = [loaded_img.width as usize, loaded_img.height as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &loaded_img.pixels);
                let texture = ctx.load_texture(
                    "loaded_image",
                    color_image,
                    egui::TextureOptions::LINEAR,
                );

                // Create project data
                let project = ProjectData::new(
                    path.to_string_lossy().to_string(),
                    loaded_img.width,
                    loaded_img.height,
                );

                self.image_texture = Some(texture);
                self.image_size = Some((loaded_img.width, loaded_img.height));
                self.project = Some(project);
                self.is_video = false;

                log::info!("Loaded image: {} ({}x{})", path.display(), loaded_img.width, loaded_img.height);
            }
            Err(e) => {
                log::error!("Failed to load image {}: {}", path.display(), e);
            }
        }
    }
}

impl eframe::App for RoidsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Image/Video...").clicked() {
                        // Open native file picker
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["jpg", "jpeg", "png", "bmp", "tiff", "tif"])
                            .pick_file()
                        {
                            self.load_image_file(path, ctx);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save Project...").clicked() {
                        // TODO: Implement save dialog
                        ui.close_menu();
                    }
                    if ui.button("Export Annotations...").clicked() {
                        // TODO: Implement export dialog
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Delete Selected").clicked() {
                        // TODO: Implement delete
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Zoom In").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Zoom Out").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Reset Zoom").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar::show(ui, &mut self.current_tool);
        });

        // Timeline (only shown for video files)
        if self.is_video {
            egui::TopBottomPanel::bottom("timeline").show(ctx, |ui| {
                timeline::show(ui);
            });
        }

        // Properties panel (right side)
        egui::SidePanel::right("properties")
            .default_width(250.0)
            .show(ctx, |ui| {
                properties::show(ui, &self.project, self.selected_annotation);
            });

        // Main canvas (center)
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas::show(
                ui,
                &self.project,
                self.current_tool,
                &self.image_texture,
                self.image_size,
            );
        });
    }
}
