use eframe::egui;

use crate::config::Config;

pub fn draw_leniency_slider(ui: &mut egui::Ui, config: &mut Config) {
    // leniency slider "centered" through the use "on the fly" slider styling and horizontal spacing
    // it is extra padded this way so that the value text box used on the slider doesn't expand the width of the program when clicked
    // egui is pain
    ui.scope(|ui| {
        let spacing = 28.0;
        ui.style_mut().spacing.slider_width = ui.available_width()-100.0- spacing*2.0;
        ui.horizontal(|ui| {
            ui.add_space(spacing);
            ui.add(egui::Slider::new(&mut config.leniency, 0.0..=99.9)
                .text("leniency")
                .clamp_to_range(true)
                .fixed_decimals(1)
            ).on_hover_text("None means any difference will be saved.\nLossless formats should probably be 0, lossy should be kept very low.");
            ui.add_space(10.0);
        });
    });
}
