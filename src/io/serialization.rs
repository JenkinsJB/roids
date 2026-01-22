// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Project data serialization and deserialization.
//!
//! This module handles exporting and importing project data in YAML
//! and JSON formats.

use crate::models::project::ProjectData;
use anyhow::Result;
use std::path::Path;

/// Export project data to YAML format with flow style for vertices.
pub fn export_yaml(data: &ProjectData, path: &Path) -> Result<()> {
    let mut yaml = serde_yaml::to_string(data)?;

    // Convert block-style vertices to flow style
    yaml = convert_vertices_to_flow_style(&yaml);

    std::fs::write(path, yaml)?;
    Ok(())
}

/// Convert block-style vertex sequences to flow style with square brackets.
fn convert_vertices_to_flow_style(yaml: &str) -> String {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check if this is a "vertices:" line
        if line.contains("vertices:") && line.trim_end().ends_with("vertices:") {
            result.push_str(line);
            result.push_str(" [");
            i += 1;

            // Collect all vertex coordinate pairs
            let mut vertices = Vec::new();
            let indent = if i < lines.len() {
                lines[i].len() - lines[i].trim_start().len()
            } else {
                0
            };

            while i < lines.len() {
                let vertex_line = lines[i];
                let current_indent = vertex_line.len() - vertex_line.trim_start().len();

                // If we've dedented or hit a non-list item, we're done with vertices
                if current_indent < indent || !vertex_line.trim_start().starts_with("- ") {
                    break;
                }

                // This is a "- -" line (start of a vertex coordinate pair)
                if vertex_line.trim_start().starts_with("- -") {
                    // Extract x coordinate
                    let x_str = vertex_line.trim_start().strip_prefix("- - ").unwrap().trim();
                    i += 1;

                    // Next line should be the y coordinate with "- "
                    if i < lines.len() {
                        let y_line = lines[i];
                        if y_line.trim_start().starts_with("- ") {
                            let y_str = y_line.trim_start().strip_prefix("- ").unwrap().trim();
                            vertices.push(format!("[{}, {}]", x_str, y_str));
                            i += 1;
                        }
                    }
                } else {
                    // Unexpected format, skip this line
                    i += 1;
                }
            }

            // Write vertices in flow style
            for (idx, vertex) in vertices.iter().enumerate() {
                if idx > 0 {
                    result.push_str(", ");
                }
                result.push_str(vertex);
            }
            result.push_str("]\n");
        } else {
            result.push_str(line);
            result.push('\n');
            i += 1;
        }
    }

    result
}

/// Export project data to JSON format.
pub fn export_json(data: &ProjectData, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Import project data from YAML format.
pub fn import_yaml(path: &Path) -> Result<ProjectData> {
    let yaml = std::fs::read_to_string(path)?;
    let data = serde_yaml::from_str(&yaml)?;
    Ok(data)
}

/// Import project data from JSON format.
pub fn import_json(path: &Path) -> Result<ProjectData> {
    let json = std::fs::read_to_string(path)?;
    let data = serde_json::from_str(&json)?;
    Ok(data)
}
