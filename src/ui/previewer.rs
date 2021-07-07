use eframe::egui::{self, Align, NumExt};

use crate::{
    app::{DEFAULT_PREVIEW_HEIGHT, LOWER_PANEL_HEIGHT, PREVIEW_IMAGE_HEIGHT},
    texture::TextureManager,
};

/// draws the image previewer that shows the before-cropped images.
pub fn draw_file_previewer(ui: &mut egui::Ui, tex_manager: &mut TextureManager) {
    if tex_manager.textures.len() == 0 {
        // small placeholder text until images are avilable
        ui.vertical_centered(|ui| {
            ui.label("(images will appear here)");
        });
    } else {
        // "add" is the space needed between here and the top of the crop bottom
        let add = (ui.clip_rect().size().y
            - ui.min_size().y
            - LOWER_PANEL_HEIGHT
            - DEFAULT_PREVIEW_HEIGHT)
            .at_least(0.0);
        egui::ScrollArea::from_max_height(DEFAULT_PREVIEW_HEIGHT + add)
            .enable_scrolling(ui.enabled())
            .show(ui, |ui| {
                // extra columns when theres enough width
                let mut columns =
                    (ui.available_width() / tex_manager.textures[0].width as f32).floor() as usize;
                if tex_manager.textures.len() < columns {
                    columns = tex_manager.textures.len();
                }
                columns = columns.at_least(3);

                egui::Grid::new("table")
                    // max and min the same, always a third of the window, min required for height limit
                    // no min row height so that they stack right on top of eachother when the window is minimized to small amounts
                    .max_col_width(ui.available_width() / columns as f32 - 4.0)
                    .min_col_width(ui.available_width() / columns as f32 - 4.0)
                    .spacing(egui::Vec2::new(5.0, 5.0))
                    .show(ui, |ui| {
                        let mut counter = 0;
                        for tex in &tex_manager.textures {
                            if counter == columns {
                                counter = 0;
                                ui.end_row();
                            }
                            counter += 1;

                            // stuff to make sure it fits nice n snug but isnt too tall or wide
                            let ratio: f32 = tex.height as f32 / tex.width as f32;
                            let predicted_height = ui.available_width() * ratio;
                            if predicted_height > PREVIEW_IMAGE_HEIGHT {
                                ui.vertical_centered(|ui| {
                                    ui.image(
                                        tex.id,
                                        egui::Vec2::new(
                                            PREVIEW_IMAGE_HEIGHT / ratio,
                                            PREVIEW_IMAGE_HEIGHT,
                                        ),
                                    );
                                });
                            } else {
                                // fit to available space only if the width is larger than the available space, otherwise just do normal texture width
                                if ui.available_width() < tex.width as f32 {
                                    ui.vertical_centered(|ui| {
                                        ui.image(
                                            tex.id,
                                            egui::Vec2::new(ui.available_width(), predicted_height),
                                        );
                                    });
                                } else {
                                    ui.vertical_centered(|ui| {
                                        ui.image(
                                            tex.id,
                                            egui::Vec2::new(tex.width as f32, tex.height as f32),
                                        );
                                    });
                                }
                            }
                        }
                    });
                // if ui is disabled we're probably loading textures, scroll to bottom to preview the new ones as they come
                if !ui.enabled() {
                    ui.scroll_to_cursor(Align::BOTTOM);
                }
            });
    }
}
