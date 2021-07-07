use eframe::egui::{self, NumExt, TextStyle};

use crate::config::{Config, NameType};

pub fn draw_filename_selector(ui: &mut egui::Ui, config: &mut Config) {
    // todo: fix this stuff up
    // yay magic width numbers because egui is pain
    let combobox_width = 120.0;
    let textbox_width = 200.0;
    let min_clip_width = 687.0;
    let padding = 16.5; // probably
    let center_padding =
        (ui.clip_rect().width() / 2.0 - (combobox_width + textbox_width + padding) / 2.0 - 3.0)
            .at_least(0.0);
    ui.horizontal(|ui| {
        if ui.clip_rect().width() < min_clip_width {
            ui.add_space(center_padding);
        }
        ui.style_mut().override_text_style = Some(TextStyle::Monospace);

        egui::ComboBox::from_id_source("bg_name")
            .selected_text(config.bg_name.name_type.name())
            .width(combobox_width)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut config.bg_name.name_type,
                    NameType::Original,
                    NameType::Original.name(),
                )
                .on_hover_text(NameType::Original.tooltip());
                ui.selectable_value(
                    &mut config.bg_name.name_type,
                    NameType::Custom,
                    NameType::Custom.name(),
                )
                .on_hover_text(NameType::Custom.tooltip());
                ui.label("test?");
            });
        let textedit = egui::TextEdit::singleline(&mut config.bg_name.name)
            .enabled(config.bg_name.name_type == NameType::Custom);
        ui.add_sized([textbox_width, 20.0], textedit)
            .on_hover_text(format!("{}.png", config.bg_name.name));
        // space between the two. yep, more magic width numbers.
        ui.add_space(ui.available_width() - textbox_width - combobox_width - padding);
        // ---
        if ui.clip_rect().width() >= min_clip_width {
            let textedit = egui::TextEdit::singleline(&mut config.file_name.name)
                .enabled(config.file_name.name_type == NameType::Custom);
            ui.add_sized([textbox_width, 20.0], textedit)
                .on_hover_text(format!("{}123.png", config.file_name.name));
            egui::ComboBox::from_id_source("file_name")
                .selected_text(config.file_name.name_type.name())
                .width(combobox_width)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut config.file_name.name_type,
                        NameType::Original,
                        NameType::Original.name(),
                    )
                    .on_hover_text(NameType::Original.tooltip());
                    ui.selectable_value(
                        &mut config.file_name.name_type,
                        NameType::Custom,
                        NameType::Custom.name(),
                    )
                    .on_hover_text(NameType::Custom.tooltip());
                });
        }
    });
    if ui.clip_rect().width() < min_clip_width {
        ui.horizontal(|ui| {
            ui.add_space(center_padding);
            egui::ComboBox::from_id_source("file_name")
                .selected_text(config.file_name.name_type.name())
                .width(combobox_width)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut config.file_name.name_type,
                        NameType::Original,
                        NameType::Original.name(),
                    )
                    .on_hover_text(NameType::Original.tooltip());
                    ui.selectable_value(
                        &mut config.file_name.name_type,
                        NameType::Custom,
                        NameType::Custom.name(),
                    )
                    .on_hover_text(NameType::Custom.tooltip());
                });
            let textedit = egui::TextEdit::singleline(&mut config.file_name.name)
                .enabled(config.file_name.name_type == NameType::Custom);
            ui.add_sized([textbox_width, 20.0], textedit)
                .on_hover_text(format!("{}123.png", config.file_name.name));
        });
    }
}
