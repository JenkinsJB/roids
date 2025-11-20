// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Media file loading (images and videos).
//!
//! This module handles loading image and video files, extracting frames,
//! and converting them to formats suitable for display in egui.

use anyhow::{Context, Result};
use image::ImageReader;
use std::path::Path;

/// Loaded image data ready for display.
pub struct LoadedImage {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// RGBA pixel data (4 bytes per pixel)
    pub pixels: Vec<u8>,
}

/// Load an image from a file path.
///
/// Supports common image formats: JPEG, PNG, BMP, TIFF, etc.
/// The image is converted to RGBA8 format for display in egui.
pub fn load_image(path: &Path) -> Result<LoadedImage> {
    // Load and decode the image
    let img = ImageReader::open(path)
        .context("Failed to open image file")?
        .decode()
        .context("Failed to decode image")?;

    // Convert to RGBA8
    let rgba_img = img.to_rgba8();
    let width = rgba_img.width();
    let height = rgba_img.height();
    let pixels = rgba_img.into_raw();

    Ok(LoadedImage {
        width,
        height,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_image_invalid_path() {
        let result = load_image(Path::new("/nonexistent/image.png"));
        assert!(result.is_err());
    }
}
