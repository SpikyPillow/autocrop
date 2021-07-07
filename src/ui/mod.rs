//! This is where I put all the groups of ui that need to be drawn, for modularity.

use eframe::egui::{self, Align, Label, Response};

use crate::{config::Config, texture::TextureManager};

pub(crate) mod croptype;
pub(crate) mod filename;
pub(crate) mod leniency;
pub(crate) mod previewer;

/// Draws a header on the left, and a browse button for something on the right.
/// Returns the response of the browse button.
pub fn label_and_browse(ui: &mut egui::Ui, label: impl Into<Label>) -> Response {
    ui.columns(2, |columns| {
        columns[0].heading(label);
        columns[1].with_layout(egui::Layout::top_down(Align::Max), |ui| ui.button("Browse"))
    })
    .inner
}

/// Draws the crop button and returns its response.
pub fn crop_button(
    ui: &mut egui::Ui,
    config: &mut Config,
    tex_manager: &mut TextureManager,
) -> Response {
    // disabled until conditions are met
    let crop_enabled =
        !(tex_manager.input_paths.len() == 0 || config.output_path.as_os_str().is_empty());
    // justified fills left and right making the button big and shiny
    ui.vertical_centered_justified(|ui| {
        let button = egui::widgets::Button::new("Crop").enabled(crop_enabled);

        ui.add_sized([0.0, 50.0], button)
            .on_disabled_hover_text("Missing output path or input images")
    })
    .inner
}
