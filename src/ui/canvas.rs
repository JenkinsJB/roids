// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Drawing canvas for image/video display and annotation.
//!
//! This module provides the main canvas area where users can view
//! images/videos and draw polygons and lines for region annotation.

use crate::app::Tool;
use crate::models::project::ProjectData;

/// Display the main canvas area.
pub fn show(ui: &mut egui::Ui, project: &Option<ProjectData>, current_tool: Tool) {
    // Set background color
    ui.style_mut().visuals.extreme_bg_color = egui::Color32::from_gray(40);

    let available_size = ui.available_size();

    // Create a frame for the canvas
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        ui.set_min_size(available_size);

        if let Some(proj) = project {
            // TODO: Draw loaded image/video frame and annotations
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Canvas: {} ({}x{})",
                        proj.media_file, proj.frame_width, proj.frame_height
                    ))
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
