// Copyright (c) 2025, Jason Jenkins
// SPDX-License-Identifier: BSD-3-Clause

//! Video timeline scrubber control.
//!
//! This module provides the timeline scrubber for navigating through
//! video frames and selecting the frame to annotate.

/// Display the timeline scrubber for video navigation.
pub fn show(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Timeline:");

        // Placeholder scrubber (will be implemented when video support is added)
        let mut frame = 0;
        ui.add(
            egui::Slider::new(&mut frame, 0..=100)
                .text("Frame")
                .show_value(true),
        );

        ui.label("/ 100");

        ui.separator();

        if ui.button("⏮").clicked() {
            // TODO: Go to first frame
        }
        if ui.button("◀").clicked() {
            // TODO: Previous frame
        }
        if ui.button("▶").clicked() {
            // TODO: Next frame
        }
        if ui.button("⏭").clicked() {
            // TODO: Go to last frame
        }
    });
}
