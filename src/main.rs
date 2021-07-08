#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// #![windows_subsystem = "windows"]

use eframe::egui::Vec2;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = autocrop::AutocropApp::new();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(500.0, 600.0)),
        minimum_window_size: Some(Vec2::new(375.0, 600.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(Box::new(app), native_options);
}
