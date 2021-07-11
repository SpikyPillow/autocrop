use eframe::egui::{self, Align, Color32, Response, Stroke, TextStyle};

use crate::config::{Config, NameType};

const COMBOBOX_WIDTH: f32 = 120.0;
const TEXTBOX_WIDTH: f32 = 200.0;

/// background image file name combo box and edit box, or normal image file name combo box and edit box
enum SelectorType {
    Background,
    Images,
}

impl SelectorType {
    fn id(&self) -> &str {
        match self {
            SelectorType::Background => "bg_name_selector",
            SelectorType::Images => "file_name_selector",
        }
    }

    fn name_type<'a>(&self, config: &'a mut Config) -> &'a mut NameType {
        match self {
            SelectorType::Background => &mut config.bg_name.name_type,
            SelectorType::Images => &mut config.file_name.name_type,
        }
    }

    fn name<'a>(&self, config: &'a mut Config) -> &'a mut String {
        match self {
            SelectorType::Background => &mut config.bg_name.name,
            SelectorType::Images => &mut config.file_name.name,
        }
    }
}

/// convenience function to draw the specific type of combo box
fn selector(ui: &mut egui::Ui, config: &mut Config, seltype: SelectorType) -> Response {
    //nametype: &mut NameType, id_source: impl std::hash::Hash
    let nametype = seltype.name_type(config);

    let hover_text = match seltype {
        SelectorType::Background => "Naming scheme for the background image.",
        SelectorType::Images => "Naming scheme for the output images.",
    };

    egui::ComboBox::from_id_source(seltype.id())
        .selected_text(nametype.name())
        .width(COMBOBOX_WIDTH)
        .show_ui(ui, |ui| {
            ui.selectable_value(nametype, NameType::Original, NameType::Original.name())
                .on_hover_text(NameType::Original.tooltip());
            ui.selectable_value(nametype, NameType::Custom, NameType::Custom.name())
                .on_hover_text(NameType::Custom.tooltip());
        })
        .on_hover_text(hover_text)
}

/// function to draw the specific edit box
fn edit_box(ui: &mut egui::Ui, config: &mut Config, seltype: SelectorType) -> Response {
    let nametype = seltype.name_type(config);
    let enabled = *nametype == NameType::Custom;

    // fighting the borrow checker
    {
        let name = seltype.name(config);

        // this isn't perfect but honestly this is too much of a pain to do properly
        // if you want better character support wait for the egui update with verticle "scrolling" text boxes
        // todo: if that ever actually happens make this not lame
        if name.chars().count() > 26 {
            *name = name.chars().take(26).collect();
        }
    }

    // technically this also blocks some non illegal names
    // but it isn't worth making seperate functionality for both types of selectors
    // todo: do it anyway
    let illegal = match seltype {
        SelectorType::Background => config.bg_name.is_illegal(),
        SelectorType::Images => config.file_name.is_illegal(),
    };

    let name = seltype.name(config);

    ui.scope(|ui| {
        if !enabled {
            ui.style_mut().visuals.override_text_color = Some(ui.style().visuals.weak_text_color());
        }
        ui.style_mut().override_text_style = Some(TextStyle::Monospace);

        let red = Color32::from_rgb(255, 0, 0);
        if illegal {
            let stroke = Stroke::new(2.0, red);
            ui.style_mut().visuals.widgets.inactive.fg_stroke = Stroke::new(5.0, red);
            ui.style_mut().visuals.selection.stroke = stroke;
            ui.style_mut().visuals.widgets.hovered.bg_stroke = stroke;
            ui.style_mut().visuals.widgets.inactive.bg_stroke = stroke;
        }

        let textedit = egui::TextEdit::singleline(name).enabled(enabled);
        let response = ui.add_sized([TEXTBOX_WIDTH, 20.0], textedit);

        // trim name on focus loss
        if response.lost_focus() {
            *name = name.trim().to_owned();
        }

        if illegal {
            egui::containers::popup::show_tooltip_under(
                &response.ctx,
                egui::Id::new("interesting thing"),
                &response.rect,
                |ui| {
                    ui.add(egui::widgets::Label::new("Invalid filename").text_color(red));
                },
            );

            response
        } else {
            // this is seperate so that we can always display the trimmed name for the tooltip
            let trim = name.trim().to_owned();

            match seltype {
                SelectorType::Background => response.on_hover_text(format!("{}.png", trim)),
                SelectorType::Images => response.on_hover_text(format!("{}123.png", trim)),
            }
        }
    })
    .inner
}

pub fn draw_filename_selector(ui: &mut egui::Ui, config: &mut Config) {
    // yay magic width numbers because egui is pain
    let min_clip_width = 700.0;
    let padding = 16.5; // probably

    if ui.clip_rect().width() >= min_clip_width {
        ui.columns(2, |columns| {
            columns[0].label("Background Image");
            columns[1].with_layout(egui::Layout::top_down(Align::Max), |ui| {
                ui.label("Other Images");
            });
        });
        ui.horizontal(|ui| {
            selector(ui, config, SelectorType::Background);
            edit_box(ui, config, SelectorType::Background);
            // ui.add_space(ui.available_width() - TEXTBOX_WIDTH - COMBOBOX_WIDTH - padding);
            ui.add_sized(
                [
                    ui.available_width() - TEXTBOX_WIDTH - COMBOBOX_WIDTH - padding - 7.0,
                    20.0,
                ],
                egui::widgets::Separator::default().horizontal(),
            );

            edit_box(ui, config, SelectorType::Images);
            selector(ui, config, SelectorType::Images);
        });
    } else {
        ui.label("Background Image");
        ui.horizontal(|ui| {
            selector(ui, config, SelectorType::Background);
            ui.add_sized(
                [ui.available_width() - TEXTBOX_WIDTH - 8.0, 20.0],
                egui::widgets::Separator::default().horizontal(),
            );
            // ui.add_space(ui.available_width()-TEXTBOX_WIDTH);
            edit_box(ui, config, SelectorType::Background);
        });
        ui.label("Other Images");
        ui.horizontal(|ui| {
            selector(ui, config, SelectorType::Images);
            ui.add_sized(
                [ui.available_width() - TEXTBOX_WIDTH - 8.0, 20.0],
                egui::widgets::Separator::default().horizontal(),
            );
            edit_box(ui, config, SelectorType::Images);
        });
    }
}
