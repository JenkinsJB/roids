// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Project data serialization and deserialization.
//!
//! This module handles exporting and importing project data in YAML
//! and JSON formats.

use crate::models::project::ProjectData;
use anyhow::Result;
use std::path::Path;

/// Export project data to YAML format.
pub fn export_yaml(data: &ProjectData, path: &Path) -> Result<()> {
    let yaml = serde_yaml::to_string(data)?;
    std::fs::write(path, yaml)?;
    Ok(())
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
