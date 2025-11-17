// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Annotation data structures.
//!
//! This module defines the core data structures for representing
//! polygons, lines, and their properties.

use serde::{Deserialize, Serialize};

/// A 2D point with normalized coordinates (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Type of annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    Polygon,
    Line,
}

/// An annotation (polygon or line) with a name and vertices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub name: String,
    #[serde(rename = "type")]
    pub annotation_type: AnnotationType,
    pub vertices: Vec<Point>,
}

impl Annotation {
    /// Create a new annotation with the given name and type.
    pub fn new(name: String, annotation_type: AnnotationType) -> Self {
        Self {
            name,
            annotation_type,
            vertices: Vec::new(),
        }
    }

    /// Add a vertex to the annotation.
    pub fn add_vertex(&mut self, point: Point) {
        self.vertices.push(point);
    }

    /// Check if the annotation is closed (polygon).
    pub fn is_closed(&self) -> bool {
        matches!(self.annotation_type, AnnotationType::Polygon)
    }
}
