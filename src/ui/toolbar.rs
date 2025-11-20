// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Toolbar and tool selection UI.
//!
//! This module provides the toolbar interface for selecting drawing
//! tools and performing common operations like file open/save.

use crate::app::Tool;

/// Display the toolbar with tool selection buttons.
pub fn show(ui: &mut egui::Ui, current_tool: &mut Tool) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        ui.label("Tools:");

        ui.separator();

        // Select tool
        if ui.selectable_label(*current_tool == Tool::Select, "⬆ Select").clicked() {
            *current_tool = Tool::Select;
        }

        // Polygon tool
        if ui.selectable_label(*current_tool == Tool::Polygon, "▱ Polygon").clicked() {
            *current_tool = Tool::Polygon;
        }

        // Line tool
        if ui.selectable_label(*current_tool == Tool::Line, "⟋ Line").clicked() {
            *current_tool = Tool::Line;
        }

        ui.separator();

        // Tool description
        let tool_text = match current_tool {
            Tool::Select => "Click to select annotations, drag vertices to move them",
            Tool::Polygon => "Click to add vertices, double-click to close polygon",
            Tool::Line => "Click to add points, press Escape to finish line",
        };

        ui.label(egui::RichText::new(tool_text).italics().weak());
    });
}
