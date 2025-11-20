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

impl Point {
    /// Create a new point with the given coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate the squared distance to another point.
    /// Using squared distance avoids expensive sqrt operation for comparisons.
    pub fn distance_squared(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Calculate the Euclidean distance to another point.
    pub fn distance(&self, other: &Point) -> f64 {
        self.distance_squared(other).sqrt()
    }
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

    /// Remove a vertex at the specified index.
    /// Returns true if a vertex was removed, false if the index was out of bounds.
    pub fn remove_vertex(&mut self, index: usize) -> bool {
        if index < self.vertices.len() {
            self.vertices.remove(index);
            true
        } else {
            false
        }
    }

    /// Find the index of the vertex closest to the given point.
    /// Returns None if the annotation has no vertices.
    pub fn find_nearest_vertex(&self, point: &Point) -> Option<usize> {
        if self.vertices.is_empty() {
            return None;
        }

        let mut min_distance = f64::MAX;
        let mut nearest_index = 0;

        for (i, vertex) in self.vertices.iter().enumerate() {
            let dist = vertex.distance_squared(point);
            if dist < min_distance {
                min_distance = dist;
                nearest_index = i;
            }
        }

        Some(nearest_index)
    }

    /// Find the vertex closest to the given point within a threshold distance.
    /// Returns None if no vertex is within the threshold.
    pub fn find_vertex_within_threshold(&self, point: &Point, threshold: f64) -> Option<usize> {
        let threshold_sq = threshold * threshold;

        self.vertices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.distance_squared(point) <= threshold_sq)
            .min_by(|(_, a), (_, b)| {
                a.distance_squared(point)
                    .partial_cmp(&b.distance_squared(point))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    /// Update the position of a vertex at the given index.
    /// Returns true if the vertex was updated, false if the index was out of bounds.
    pub fn update_vertex(&mut self, index: usize, new_position: Point) -> bool {
        if index < self.vertices.len() {
            self.vertices[index] = new_position;
            true
        } else {
            false
        }
    }

    /// Check if the annotation is closed (polygon).
    pub fn is_closed(&self) -> bool {
        matches!(self.annotation_type, AnnotationType::Polygon)
    }

    /// Get the number of vertices in this annotation.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let point = Point::new(0.5, 0.75);
        assert_eq!(point.x, 0.5);
        assert_eq!(point.y, 0.75);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);

        assert_eq!(p1.distance(&p2), 5.0);
        assert_eq!(p1.distance_squared(&p2), 25.0);
    }

    #[test]
    fn test_annotation_new() {
        let annotation = Annotation::new("region 1".to_string(), AnnotationType::Polygon);
        assert_eq!(annotation.name, "region 1");
        assert_eq!(annotation.annotation_type, AnnotationType::Polygon);
        assert_eq!(annotation.vertices.len(), 0);
        assert!(annotation.is_closed());
    }

    #[test]
    fn test_annotation_add_vertex() {
        let mut annotation = Annotation::new("line 1".to_string(), AnnotationType::Line);
        annotation.add_vertex(Point::new(0.0, 0.0));
        annotation.add_vertex(Point::new(1.0, 1.0));

        assert_eq!(annotation.vertex_count(), 2);
        assert!(!annotation.is_closed());
    }

    #[test]
    fn test_annotation_remove_vertex() {
        let mut annotation = Annotation::new("region 1".to_string(), AnnotationType::Polygon);
        annotation.add_vertex(Point::new(0.0, 0.0));
        annotation.add_vertex(Point::new(1.0, 0.0));
        annotation.add_vertex(Point::new(1.0, 1.0));

        assert!(annotation.remove_vertex(1));
        assert_eq!(annotation.vertex_count(), 2);
        assert_eq!(annotation.vertices[1], Point::new(1.0, 1.0));

        assert!(!annotation.remove_vertex(10));
    }

    #[test]
    fn test_annotation_update_vertex() {
        let mut annotation = Annotation::new("region 1".to_string(), AnnotationType::Polygon);
        annotation.add_vertex(Point::new(0.0, 0.0));
        annotation.add_vertex(Point::new(1.0, 0.0));

        assert!(annotation.update_vertex(0, Point::new(0.5, 0.5)));
        assert_eq!(annotation.vertices[0], Point::new(0.5, 0.5));

        assert!(!annotation.update_vertex(10, Point::new(0.0, 0.0)));
    }

    #[test]
    fn test_find_nearest_vertex() {
        let mut annotation = Annotation::new("region 1".to_string(), AnnotationType::Polygon);
        annotation.add_vertex(Point::new(0.0, 0.0));
        annotation.add_vertex(Point::new(1.0, 0.0));
        annotation.add_vertex(Point::new(1.0, 1.0));

        let search_point = Point::new(0.95, 0.05);
        let nearest = annotation.find_nearest_vertex(&search_point);
        assert_eq!(nearest, Some(1));

        let empty_annotation = Annotation::new("empty".to_string(), AnnotationType::Line);
        assert_eq!(empty_annotation.find_nearest_vertex(&search_point), None);
    }

    #[test]
    fn test_find_vertex_within_threshold() {
        let mut annotation = Annotation::new("region 1".to_string(), AnnotationType::Polygon);
        annotation.add_vertex(Point::new(0.0, 0.0));
        annotation.add_vertex(Point::new(0.5, 0.0));
        annotation.add_vertex(Point::new(1.0, 0.0));

        let search_point = Point::new(0.52, 0.02);
        let found = annotation.find_vertex_within_threshold(&search_point, 0.05);
        assert_eq!(found, Some(1));

        let found_none = annotation.find_vertex_within_threshold(&search_point, 0.01);
        assert_eq!(found_none, None);
    }

    #[test]
    fn test_serialization() {
        let mut annotation = Annotation::new("test region".to_string(), AnnotationType::Polygon);
        annotation.add_vertex(Point::new(0.1, 0.2));
        annotation.add_vertex(Point::new(0.3, 0.4));

        let json = serde_json::to_string(&annotation).unwrap();
        let deserialized: Annotation = serde_json::from_str(&json).unwrap();

        assert_eq!(annotation, deserialized);
    }
}
