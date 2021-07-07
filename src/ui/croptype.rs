use eframe::egui;

use crate::config::{Config, CropType};

/// Draws the croptype selector and the "resize output" to the right of it.
pub fn draw_croptype_selector(ui: &mut egui::Ui, config: &mut Config) {
    // centering is stupid, just stop thinking about it
    ui.horizontal(|ui| {
        ui.add_space(28.0);
        egui::ComboBox::from_label("Crop Type")
            .selected_text(config.crop_type.name())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut config.crop_type,
                    CropType::Rectangle,
                    CropType::Rectangle.name(),
                )
                .on_hover_text(CropType::Rectangle.tooltip());
                ui.selectable_value(
                    &mut config.crop_type,
                    CropType::Exact,
                    CropType::Exact.name(),
                )
                .on_hover_text(CropType::Exact.tooltip());
            });
        ui.add_space(ui.available_width() - 118.0);
        ui.checkbox(&mut config.resize_output, "resize output")
            .on_hover_text("When false, cropped out space\nis replaced with empty pixels.");
    });
}
