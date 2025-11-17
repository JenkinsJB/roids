// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Geometric utility functions.
//!
//! This module provides utilities for coordinate transformations between
//! pixel coordinates and normalized coordinates.

use crate::models::annotation::Point;

/// Convert pixel coordinates to normalized coordinates (0.0 to 1.0).
pub fn normalize_coordinates(pixel_x: f64, pixel_y: f64, width: u32, height: u32) -> Point {
    Point {
        x: pixel_x / width as f64,
        y: pixel_y / height as f64,
    }
}

/// Convert normalized coordinates to pixel coordinates.
pub fn denormalize_coordinates(point: &Point, width: u32, height: u32) -> (f64, f64) {
    (point.x * width as f64, point.y * height as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_denormalize_roundtrip() {
        let width = 1920;
        let height = 1080;
        let pixel_x = 960.0;
        let pixel_y = 540.0;

        let normalized = normalize_coordinates(pixel_x, pixel_y, width, height);
        let (denorm_x, denorm_y) = denormalize_coordinates(&normalized, width, height);

        assert!((denorm_x - pixel_x).abs() < 0.0001);
        assert!((denorm_y - pixel_y).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_corners() {
        let width = 1920;
        let height = 1080;

        // Top-left corner
        let tl = normalize_coordinates(0.0, 0.0, width, height);
        assert_eq!(tl.x, 0.0);
        assert_eq!(tl.y, 0.0);

        // Bottom-right corner
        let br = normalize_coordinates(1920.0, 1080.0, width, height);
        assert_eq!(br.x, 1.0);
        assert_eq!(br.y, 1.0);
    }
}
