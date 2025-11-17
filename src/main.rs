// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! ROIDS - Region Of Interest Designation System
//!
//! A cross-platform desktop application for annotating images and videos
//! with regions of interest (polygons) and counting lines.

mod app;
mod io;
mod models;
mod ui;
mod util;

use app::RoidsApp;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Configure egui options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ROIDS - Region Of Interest Designation System"),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "ROIDS",
        options,
        Box::new(|_cc| Ok(Box::new(RoidsApp::new()))),
    )
    .map_err(|e| anyhow::anyhow!("Application error: {}", e))?;

    Ok(())
}
