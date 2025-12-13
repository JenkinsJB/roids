// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Main application state and egui App implementation.
//!
//! This module contains the main application structure that implements
//! the egui::App trait, managing the overall application state and
//! coordinating between different UI components and the data model.

use crate::models::{
    annotation::{Annotation, AnnotationType},
    project::ProjectData,
};
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

    /// In-progress annotation being drawn
    in_progress_annotation: Option<Annotation>,

    /// Counter for generating default annotation names
    annotation_counter: usize,
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
            in_progress_annotation: None,
            annotation_counter: 0,
        }
    }

    /// Start a new annotation based on the current tool.
    fn start_annotation(&mut self) {
        let annotation_type = match self.current_tool {
            Tool::Polygon => AnnotationType::Polygon,
            Tool::Line => AnnotationType::Line,
            Tool::Select => return, // Don't create annotations in select mode
        };

        let name = match annotation_type {
            AnnotationType::Polygon => format!("region {}", self.annotation_counter + 1),
            AnnotationType::Line => format!("line {}", self.annotation_counter + 1),
        };

        self.in_progress_annotation = Some(Annotation::new(name, annotation_type));
    }

    /// Finish the current in-progress annotation and add it to the project.
    fn finish_annotation(&mut self) {
        if let Some(annotation) = self.in_progress_annotation.take() {
            if annotation.vertex_count() >= 2 {
                if let Some(ref mut project) = self.project {
                    project.annotations.push(annotation);
                    self.annotation_counter += 1;
                    log::info!("Added annotation, total: {}", project.annotations.len());
                }
            }
        }
    }

    /// Cancel the current in-progress annotation.
    fn cancel_annotation(&mut self) {
        self.in_progress_annotation = None;
    }

    /// Export annotations to a file.
    fn export_annotations(&self, path: std::path::PathBuf) {
        if let Some(ref project) = self.project {
            let extension = path.extension().and_then(|s| s.to_str());
            let result = match extension {
                Some("yaml") | Some("yml") => crate::io::serialization::export_yaml(project, &path),
                Some("json") => crate::io::serialization::export_json(project, &path),
                _ => {
                    log::error!("Unsupported file extension: {:?}", extension);
                    return;
                }
            };

            match result {
                Ok(_) => log::info!("Exported annotations to {}", path.display()),
                Err(e) => log::error!("Failed to export annotations: {}", e),
            }
        }
    }

    /// Import annotations from a file and load the associated image.
    fn import_annotations(&mut self, path: std::path::PathBuf, ctx: &egui::Context) {
        let extension = path.extension().and_then(|s| s.to_str());
        let result = match extension {
            Some("yaml") | Some("yml") => crate::io::serialization::import_yaml(&path),
            Some("json") => crate::io::serialization::import_json(&path),
            _ => {
                log::error!("Unsupported file extension: {:?}", extension);
                return;
            }
        };

        match result {
            Ok(project_data) => {
                log::info!("Imported {} annotations from {}",
                    project_data.annotations.len(), path.display());

                // Try to load the referenced image file
                let image_path = std::path::PathBuf::from(&project_data.media_file);
                if image_path.exists() {
                    match crate::io::media::load_image(&image_path) {
                        Ok(loaded_img) => {
                            // Create egui texture from the loaded image
                            let size = [loaded_img.width as usize, loaded_img.height as usize];
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &loaded_img.pixels);
                            let texture = ctx.load_texture(
                                "loaded_image",
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );

                            self.image_texture = Some(texture);
                            self.image_size = Some((loaded_img.width, loaded_img.height));
                            log::info!("Loaded image: {}", image_path.display());
                        }
                        Err(e) => {
                            log::error!("Failed to load image {}: {}", image_path.display(), e);
                        }
                    }
                } else {
                    log::warn!("Referenced image not found: {}", image_path.display());
                }

                // Update annotation counter based on loaded annotations
                self.annotation_counter = project_data.annotations.len();
                self.project = Some(project_data);
            }
            Err(e) => log::error!("Failed to import annotations: {}", e),
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
                    if ui.button("Load Annotations...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Annotations", &["yaml", "yml", "json"])
                            .pick_file()
                        {
                            self.import_annotations(path, ctx);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Export Annotations", |ui| {
                        if ui.button("Export as YAML...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("YAML", &["yaml", "yml"])
                                .set_file_name("annotations.yaml")
                                .save_file()
                            {
                                self.export_annotations(path);
                            }
                            ui.close_menu();
                        }
                        if ui.button("Export as JSON...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("annotations.json")
                                .save_file()
                            {
                                self.export_annotations(path);
                            }
                            ui.close_menu();
                        }
                    });
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

        // Handle keyboard events
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.current_tool == Tool::Line && self.in_progress_annotation.is_some() {
                // Finish line on Escape
                self.finish_annotation();
            } else {
                // Cancel annotation on Escape
                self.cancel_annotation();
            }
        }

        // Main canvas (center)
        let canvas_action = egui::CentralPanel::default().show(ctx, |ui| {
            canvas::show(
                ui,
                &self.project,
                self.current_tool,
                &self.image_texture,
                self.image_size,
                &self.in_progress_annotation,
            )
        }).inner;

        // Handle canvas actions
        match canvas_action {
            canvas::CanvasAction::AddVertex(point) => {
                // Start new annotation if none in progress
                if self.in_progress_annotation.is_none() {
                    self.start_annotation();
                }

                // Add vertex to in-progress annotation
                if let Some(ref mut annotation) = self.in_progress_annotation {
                    annotation.add_vertex(point);
                    log::info!("Added vertex at ({:.3}, {:.3}), total vertices: {}",
                        point.x, point.y, annotation.vertex_count());
                }
            }
            canvas::CanvasAction::FinishAnnotation => {
                // Finish the annotation (for double-click on polygon)
                self.finish_annotation();
            }
            canvas::CanvasAction::None => {}
        }
    }
}
