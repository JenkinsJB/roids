// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Project state management.
//!
//! This module manages the overall project state including loaded media,
//! annotations, and application settings.

use super::annotation::Annotation;
use serde::{Deserialize, Serialize};

/// Complete project data for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub media_file: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub annotations: Vec<Annotation>,
}

impl ProjectData {
    /// Create a new project with the given media file and dimensions.
    pub fn new(media_file: String, frame_width: u32, frame_height: u32) -> Self {
        Self {
            media_file,
            frame_width,
            frame_height,
            annotations: Vec::new(),
        }
    }
}
