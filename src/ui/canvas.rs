// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Drawing canvas for image/video display and annotation.
//!
//! This module provides the main canvas area where users can view
//! images/videos and draw polygons and lines for region annotation.

use crate::app::Tool;
use crate::models::{annotation::{Annotation, Point}, project::ProjectData};

/// Result of canvas interaction.
pub enum CanvasAction {
    None,
    AddVertex(Point),
    FinishAnnotation,
}

/// Display the main canvas area and handle mouse interactions.
pub fn show(
    ui: &mut egui::Ui,
    project: &Option<ProjectData>,
    current_tool: Tool,
    image_texture: &Option<egui::TextureHandle>,
    image_size: Option<(u32, u32)>,
    in_progress_annotation: &Option<Annotation>,
) -> CanvasAction {
    let mut action = CanvasAction::None;
    // Set background color
    ui.style_mut().visuals.extreme_bg_color = egui::Color32::from_gray(40);

    let available_size = ui.available_size();

    // Create a frame for the canvas
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        ui.set_min_size(available_size);

        if let Some(texture) = image_texture {
            // Display the loaded image
            if let Some((img_width, img_height)) = image_size {
                // Calculate scaling to fit the image in the available space
                let available = ui.available_size();
                let img_aspect = img_width as f32 / img_height as f32;
                let available_aspect = available.x / available.y;

                let (display_width, display_height) = if img_aspect > available_aspect {
                    // Image is wider - fit to width
                    let width = available.x;
                    let height = width / img_aspect;
                    (width, height)
                } else {
                    // Image is taller - fit to height
                    let height = available.y;
                    let width = height * img_aspect;
                    (width, height)
                };

                // Center the image
                let x_offset = (available.x - display_width) / 2.0;
                let y_offset = (available.y - display_height) / 2.0;

                let image_rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2(x_offset, y_offset),
                    egui::vec2(display_width, display_height),
                );

                // Draw the image
                ui.painter().image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // Handle mouse interactions for drawing tools
                if current_tool != Tool::Select {
                    let response = ui.allocate_rect(image_rect, egui::Sense::click());

                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            // Convert screen coordinates to normalized coordinates
                            if image_rect.contains(pos) {
                                let rel_x = (pos.x - image_rect.min.x) / display_width;
                                let rel_y = (pos.y - image_rect.min.y) / display_height;
                                action = CanvasAction::AddVertex(Point::new(
                                    rel_x as f64,
                                    rel_y as f64,
                                ));
                            }
                        }
                    }

                    if response.double_clicked() && current_tool == Tool::Polygon {
                        action = CanvasAction::FinishAnnotation;
                    }
                }

                // Draw annotations on top of the image
                let painter = ui.painter();

                // Draw completed annotations
                if let Some(proj) = project {
                    for annotation in &proj.annotations {
                        draw_annotation(painter, annotation, &image_rect, egui::Color32::YELLOW, false);
                    }
                }

                // Draw in-progress annotation
                if let Some(annotation) = in_progress_annotation {
                    draw_annotation(painter, annotation, &image_rect, egui::Color32::LIGHT_BLUE, true);
                }
            }
        } else if project.is_some() {
            // Project loaded but no image texture (shouldn't happen normally)
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Loading image...")
                        .color(egui::Color32::WHITE),
                );
            });
        } else {
            // Show welcome message when no file is loaded
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.heading(
                        egui::RichText::new("ROIDS")
                            .size(32.0)
                            .color(egui::Color32::from_gray(200)),
                    );
                    ui.label(
                        egui::RichText::new("Region Of Interest Designation System")
                            .size(14.0)
                            .color(egui::Color32::from_gray(150)),
                    );
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new("Open an image or video to begin annotating")
                            .color(egui::Color32::from_gray(180)),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("File → Open Image/Video...")
                            .weak()
                            .color(egui::Color32::from_gray(130)),
                    );
                });
            });
        }
    });

    // Display current tool info at the bottom
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(format!("Current tool: {:?}", current_tool));
        if project.is_some() {
            ui.separator();
            ui.label("Ready");
        } else {
            ui.separator();
            ui.label("No file loaded");
        }
    });

    action
}

/// Draw an annotation on the canvas.
fn draw_annotation(
    painter: &egui::Painter,
    annotation: &Annotation,
    image_rect: &egui::Rect,
    color: egui::Color32,
    is_in_progress: bool,
) {
    let vertices = &annotation.vertices;
    if vertices.is_empty() {
        return;
    }

    // Convert normalized coordinates to screen coordinates
    let screen_points: Vec<egui::Pos2> = vertices
        .iter()
        .map(|p| {
            egui::pos2(
                image_rect.min.x + (p.x as f32) * image_rect.width(),
                image_rect.min.y + (p.y as f32) * image_rect.height(),
            )
        })
        .collect();

    // Draw lines connecting vertices
    for i in 0..screen_points.len() {
        let next_i = (i + 1) % screen_points.len();

        // For in-progress annotations, don't connect last vertex back to first
        if is_in_progress && next_i == 0 {
            break;
        }

        // For closed polygons, draw all edges including back to first
        if !is_in_progress || i < screen_points.len() - 1 {
            painter.line_segment(
                [screen_points[i], screen_points[next_i]],
                egui::Stroke::new(2.0, color),
            );
        }
    }

    // Draw vertices as circles
    let vertex_color = if is_in_progress {
        egui::Color32::WHITE
    } else {
        color
    };

    for point in &screen_points {
        painter.circle_filled(*point, 4.0, vertex_color);
        painter.circle_stroke(*point, 4.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
    }
}
