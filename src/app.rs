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
use crate::ui::{canvas, properties, toolbar};
use std::sync::mpsc::{channel, Receiver};

/// History system for undo/redo functionality.
struct History {
    /// Undo stack (past states)
    undo_stack: Vec<Vec<Annotation>>,
    /// Redo stack (future states after undo)
    redo_stack: Vec<Vec<Annotation>>,
    /// Maximum history size
    max_size: usize,
}

impl History {
    fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 50, // Keep last 50 states
        }
    }

    /// Save current state before making a change
    fn push(&mut self, annotations: Vec<Annotation>) {
        self.undo_stack.push(annotations);
        // Limit history size
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
        // Clear redo stack when new action is performed
        self.redo_stack.clear();
    }

    /// Undo: restore previous state
    fn undo(&mut self, current: Vec<Annotation>) -> Option<Vec<Annotation>> {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(previous)
        } else {
            None
        }
    }

    /// Redo: restore next state
    fn redo(&mut self, current: Vec<Annotation>) -> Option<Vec<Annotation>> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }

    /// Check if undo is available
    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// Current drawing tool selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Polygon,
    Line,
}

/// Result of background image loading operation.
struct LoadedImageData {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    project: Option<ProjectData>,
}

/// Main application state.
pub struct RoidsApp {
    /// Currently selected drawing tool
    current_tool: Tool,

    /// Current project data (if a file is loaded)
    project: Option<ProjectData>,

    /// Index of currently selected annotation
    selected_annotation: Option<usize>,

    /// Loaded image texture for display
    image_texture: Option<egui::TextureHandle>,

    /// Image dimensions (width, height)
    image_size: Option<(u32, u32)>,

    /// In-progress annotation being drawn
    in_progress_annotation: Option<Annotation>,

    /// Counter for generating default annotation names
    annotation_counter: usize,

    /// Currently dragged vertex (annotation_index, vertex_index)
    dragging_vertex: Option<(usize, usize)>,

    /// History for undo/redo
    history: History,

    /// Receiver for background image loading
    image_loader: Option<Receiver<Result<LoadedImageData, String>>>,

    /// Loading state message
    loading_message: Option<String>,
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
            image_texture: None,
            image_size: None,
            in_progress_annotation: None,
            annotation_counter: 0,
            dragging_vertex: None,
            history: History::new(),
            image_loader: None,
            loading_message: None,
        }
    }

    /// Save annotations to history before making a change
    fn save_to_history(&mut self, annotations: &[Annotation]) {
        self.history.push(annotations.to_vec());
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
                // Clone annotations for history
                let annotations_clone = self.project.as_ref()
                    .map(|p| p.annotations.clone());

                // Save to history before making changes
                if let Some(annotations) = annotations_clone {
                    self.save_to_history(&annotations);
                }

                // Now mutably borrow and make changes
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

    /// Import annotations from a file and load the associated image (asynchronously).
    fn import_annotations(&mut self, path: std::path::PathBuf, _ctx: &egui::Context) {
        let (sender, receiver) = channel();
        self.image_loader = Some(receiver);
        self.loading_message = Some("Loading annotations and image...".to_string());

        // Spawn background thread for loading
        std::thread::spawn(move || {
            let result = (|| -> Result<LoadedImageData, String> {
                // Parse annotation file
                let extension = path.extension().and_then(|s| s.to_str());
                let project_data = match extension {
                    Some("yaml") | Some("yml") => crate::io::serialization::import_yaml(&path)
                        .map_err(|e| format!("Failed to import YAML: {}", e))?,
                    Some("json") => crate::io::serialization::import_json(&path)
                        .map_err(|e| format!("Failed to import JSON: {}", e))?,
                    _ => return Err(format!("Unsupported file extension: {:?}", extension)),
                };

                log::info!("Imported {} annotations from {}",
                    project_data.annotations.len(), path.display());

                // Load the referenced image file
                let image_path = std::path::PathBuf::from(&project_data.media_file);
                if !image_path.exists() {
                    return Err(format!("Referenced image not found: {}", image_path.display()));
                }

                let loaded_img = crate::io::media::load_image(&image_path)
                    .map_err(|e| format!("Failed to load image: {}", e))?;

                log::info!("Loaded image: {}", image_path.display());

                Ok(LoadedImageData {
                    width: loaded_img.width,
                    height: loaded_img.height,
                    pixels: loaded_img.pixels,
                    project: Some(project_data),
                })
            })();

            let _ = sender.send(result);
        });
    }

    /// Load an image file and create a texture for display (asynchronously).
    pub fn load_image_file(&mut self, path: std::path::PathBuf, _ctx: &egui::Context) {
        let (sender, receiver) = channel();
        self.image_loader = Some(receiver);
        self.loading_message = Some("Loading image...".to_string());

        let path_string = path.to_string_lossy().to_string();

        // Spawn background thread for loading
        std::thread::spawn(move || {
            let result = (|| -> Result<LoadedImageData, String> {
                let loaded_img = crate::io::media::load_image(&path)
                    .map_err(|e| format!("Failed to load image: {}", e))?;

                log::info!("Loaded image: {} ({}x{})", path.display(), loaded_img.width, loaded_img.height);

                // Create project data
                let project = ProjectData::new(
                    path_string,
                    loaded_img.width,
                    loaded_img.height,
                );

                Ok(LoadedImageData {
                    width: loaded_img.width,
                    height: loaded_img.height,
                    pixels: loaded_img.pixels,
                    project: Some(project),
                })
            })();

            let _ = sender.send(result);
        });
    }
}

impl eframe::App for RoidsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for completed image loading
        if let Some(ref receiver) = self.image_loader {
            if let Ok(result) = receiver.try_recv() {
                self.image_loader = None;
                self.loading_message = None;

                match result {
                    Ok(loaded_data) => {
                        // Create egui texture from the loaded image data
                        let size = [loaded_data.width as usize, loaded_data.height as usize];
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &loaded_data.pixels);
                        let texture = ctx.load_texture(
                            "loaded_image",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );

                        self.image_texture = Some(texture);
                        self.image_size = Some((loaded_data.width, loaded_data.height));

                        if let Some(project) = loaded_data.project {
                            // Update annotation counter based on loaded annotations
                            self.annotation_counter = project.annotations.len();
                            self.project = Some(project);
                        }

                        log::info!("Image loaded successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to load image: {}", e);
                    }
                }
            }
        }

        // Request repaint if still loading (to update spinner)
        if self.loading_message.is_some() {
            ctx.request_repaint();
        }

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Image...").clicked() {
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
                    // Undo
                    let can_undo = self.history.can_undo();
                    if ui.add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)")).clicked() {
                        if let Some(ref mut project) = self.project {
                            let current = project.annotations.clone();
                            if let Some(previous) = self.history.undo(current) {
                                project.annotations = previous;
                                self.selected_annotation = None;
                                log::info!("Undo from menu");
                            }
                        }
                        ui.close_menu();
                    }

                    // Redo
                    let can_redo = self.history.can_redo();
                    if ui.add_enabled(can_redo, egui::Button::new("Redo (Ctrl+Shift+Z)")).clicked() {
                        if let Some(ref mut project) = self.project {
                            let current = project.annotations.clone();
                            if let Some(next) = self.history.redo(current) {
                                project.annotations = next;
                                self.selected_annotation = None;
                                log::info!("Redo from menu");
                            }
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    // Delete Selected
                    let has_selection = self.selected_annotation.is_some();
                    if ui.add_enabled(has_selection, egui::Button::new("Delete Selected")).clicked() {
                        if let Some(idx) = self.selected_annotation {
                            // Clone annotations for history
                            let annotations_clone = self.project.as_ref()
                                .filter(|p| idx < p.annotations.len())
                                .map(|p| p.annotations.clone());

                            // Save to history before making changes
                            if let Some(annotations) = annotations_clone {
                                self.save_to_history(&annotations);
                            }

                            // Now mutably borrow and make changes
                            if let Some(ref mut project) = self.project {
                                if idx < project.annotations.len() {
                                    project.annotations.remove(idx);
                                    self.selected_annotation = None;
                                    log::info!("Deleted annotation from menu, total: {}", project.annotations.len());
                                }
                            }
                        }
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

        // Properties panel (right side)
        let properties_action = egui::SidePanel::right("properties")
            .default_width(250.0)
            .show(ctx, |ui| {
                properties::show(ui, &mut self.project, self.selected_annotation)
            }).inner;

        // Handle properties panel actions
        match properties_action {
            properties::PropertiesAction::SelectAnnotation(idx) => {
                self.selected_annotation = Some(idx);
            }
            properties::PropertiesAction::DeleteAnnotation(idx) => {
                // Clone annotations for history
                let annotations_clone = self.project.as_ref()
                    .filter(|p| idx < p.annotations.len())
                    .map(|p| p.annotations.clone());

                // Save to history before making changes
                if let Some(annotations) = annotations_clone {
                    self.save_to_history(&annotations);
                }

                // Now mutably borrow and make changes
                if let Some(ref mut project) = self.project {
                    if idx < project.annotations.len() {
                        project.annotations.remove(idx);
                        self.selected_annotation = None;
                        log::info!("Deleted annotation from panel, total: {}", project.annotations.len());
                    }
                }
            }
            properties::PropertiesAction::None => {}
        }

        // Handle keyboard events
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.current_tool == Tool::Line && self.in_progress_annotation.is_some() {
                // Finish line on Escape
                self.finish_annotation();
            } else {
                // Cancel annotation on Escape or deselect
                self.cancel_annotation();
                self.selected_annotation = None;
            }
        }

        // Handle Delete key to delete selected annotation
        // Only process if no text field is focused (to avoid deleting while editing names)
        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                if let Some(idx) = self.selected_annotation {
                    // Clone annotations for history
                    let annotations_clone = self.project.as_ref()
                        .filter(|p| idx < p.annotations.len())
                        .map(|p| p.annotations.clone());

                    // Save to history before making changes
                    if let Some(annotations) = annotations_clone {
                        self.save_to_history(&annotations);
                    }

                    // Now mutably borrow and make changes
                    if let Some(ref mut project) = self.project {
                        if idx < project.annotations.len() {
                            project.annotations.remove(idx);
                            self.selected_annotation = None;
                            log::info!("Deleted annotation, total: {}", project.annotations.len());
                        }
                    }
                }
            }

            // Handle undo (Ctrl+Z)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift) {
                if self.history.can_undo() {
                    if let Some(ref mut project) = self.project {
                        let current = project.annotations.clone();
                        if let Some(previous) = self.history.undo(current) {
                            project.annotations = previous;
                            self.selected_annotation = None;
                            log::info!("Undo");
                        }
                    }
                }
            }

            // Handle redo (Ctrl+Shift+Z or Ctrl+Y)
            if ctx.input(|i| {
                (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)) ||
                (i.modifiers.command && i.key_pressed(egui::Key::Y))
            }) {
                if self.history.can_redo() {
                    if let Some(ref mut project) = self.project {
                        let current = project.annotations.clone();
                        if let Some(next) = self.history.redo(current) {
                            project.annotations = next;
                            self.selected_annotation = None;
                            log::info!("Redo");
                        }
                    }
                }
            }
        }

        // Main canvas (center)
        let canvas_action = egui::CentralPanel::default().show(ctx, |ui| {
            // Show loading overlay if loading
            if let Some(ref message) = self.loading_message {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(message)
                                .size(16.0)
                                .color(egui::Color32::from_gray(200)),
                        );
                    });
                });
                canvas::CanvasAction::None
            } else {
                canvas::show(
                    ui,
                    &self.project,
                    self.current_tool,
                    &self.image_texture,
                    self.image_size,
                    &self.in_progress_annotation,
                    self.selected_annotation,
                    self.dragging_vertex,
                )
            }
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
            canvas::CanvasAction::SelectAnnotation(idx) => {
                self.selected_annotation = Some(idx);
                log::info!("Selected annotation {}", idx);
            }
            canvas::CanvasAction::DeselectAnnotation => {
                self.selected_annotation = None;
                log::info!("Deselected annotation");
            }
            canvas::CanvasAction::StartDraggingVertex(ann_idx, vertex_idx) => {
                // Clone annotations for history
                let annotations_clone = self.project.as_ref()
                    .map(|p| p.annotations.clone());

                // Save to history before starting drag
                if let Some(annotations) = annotations_clone {
                    self.save_to_history(&annotations);
                }

                self.dragging_vertex = Some((ann_idx, vertex_idx));
                self.selected_annotation = Some(ann_idx);
                log::info!("Started dragging vertex {} of annotation {}", vertex_idx, ann_idx);
            }
            canvas::CanvasAction::DragVertex(point) => {
                if let Some((ann_idx, vertex_idx)) = self.dragging_vertex {
                    if let Some(ref mut project) = self.project {
                        if let Some(annotation) = project.annotations.get_mut(ann_idx) {
                            annotation.update_vertex(vertex_idx, point);
                        }
                    }
                }
            }
            canvas::CanvasAction::StopDragging => {
                if let Some((ann_idx, vertex_idx)) = self.dragging_vertex {
                    log::info!("Stopped dragging vertex {} of annotation {}", vertex_idx, ann_idx);
                }
                self.dragging_vertex = None;
            }
            canvas::CanvasAction::None => {}
        }
    }
}
