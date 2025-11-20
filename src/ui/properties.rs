// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Annotation properties panel.
//!
//! This module provides the properties panel for viewing and editing
//! annotation metadata such as names, types, and vertex coordinates.

use crate::models::project::ProjectData;

/// Display the properties panel showing annotations and their details.
pub fn show(
    ui: &mut egui::Ui,
    project: &Option<ProjectData>,
    selected_annotation: Option<usize>,
) {
    ui.heading("Annotations");
    ui.separator();

    if let Some(proj) = project {
        if proj.annotations.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("No annotations yet")
                        .weak()
                        .italics(),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Use the Polygon or Line tool\nto create annotations")
                        .weak()
                        .small(),
                );
            });
        } else {
            // List all annotations
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, annotation) in proj.annotations.iter().enumerate() {
                    let is_selected = selected_annotation == Some(i);

                    ui.horizontal(|ui| {
                        let label_text = format!(
                            "{} ({} vertices)",
                            annotation.name,
                            annotation.vertex_count()
                        );

                        if ui.selectable_label(is_selected, label_text).clicked() {
                            // TODO: Select this annotation
                        }
                    });

                    // Show details if selected
                    if is_selected {
                        ui.indent(format!("annotation_{}", i), |ui| {
                            ui.label(format!("Type: {:?}", annotation.annotation_type));
                            ui.label(format!("Vertices: {}", annotation.vertex_count()));

                            if ui.button("Delete").clicked() {
                                // TODO: Delete annotation
                            }
                        });
                    }
                }
            });
        }
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(
                egui::RichText::new("No file loaded")
                    .weak()
                    .italics(),
            );
        });
    }

    ui.separator();

    // Properties section
    if let Some(idx) = selected_annotation {
        if let Some(proj) = project {
            if let Some(annotation) = proj.annotations.get(idx) {
                ui.heading("Properties");
                ui.separator();

                ui.label(format!("Name: {}", annotation.name));
                ui.label(format!("Type: {:?}", annotation.annotation_type));
                ui.label(format!("Closed: {}", annotation.is_closed()));
                ui.label(format!("Vertices: {}", annotation.vertex_count()));

                // TODO: Add editable properties
            }
        }
    }
}
