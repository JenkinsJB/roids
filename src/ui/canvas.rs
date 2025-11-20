// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Drawing canvas for image/video display and annotation.
//!
//! This module provides the main canvas area where users can view
//! images/videos and draw polygons and lines for region annotation.

use crate::app::Tool;
use crate::models::project::ProjectData;

/// Display the main canvas area.
pub fn show(
    ui: &mut egui::Ui,
    project: &Option<ProjectData>,
    current_tool: Tool,
    image_texture: &Option<egui::TextureHandle>,
    image_size: Option<(u32, u32)>,
) {
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

                // TODO: Draw annotations on top of the image
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
}
