use std::{
    path::PathBuf,
    str::FromStr,
    sync::mpsc::{self, Receiver, Sender},
};

use eframe::{
    egui::{self, Align, NumExt, TextStyle},
    epi,
};
use image::DynamicImage;
use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::config::Config;
use crate::{
    config::{CropType, NameType},
    texture::TextureManager,
};

/// crate version, for the display on the bottom right
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
/// set max height the images in the previewer will be, this is what height resizes to if it is greater than width
pub const PREVIEW_IMAGE_HEIGHT: f32 = 250.0;
/// set max width the images in the previewer will be, this is what width resizes to if it is greater than height
pub const PREVIEW_IMAGE_WIDTH: f32 = 300.0;
/// max amount of preview images before program stops generating previews to save ram
pub const PREVEW_IMAGE_LIMIT: usize = 1000;
/// the minimum height for the previewer ui itself
const DEFAULT_PREVIEW_HEIGHT: f32 = 200.0;
/// the fake "lower panel" (crop button) height. hard coded because pain.
const LOWER_PANEL_HEIGHT: f32 = 75.0;
/// the lower panel height to adjust for to make the scrollbar not appear longer than it should.
const SCROLLBAR_ADJUST: f32 = 35.0;

#[derive(Default)]
pub struct AutocropApp {
    tex_manager: TextureManager,
    reciever: Option<Receiver<DynamicImage>>,
    config: Config,
}

impl AutocropApp {
    pub fn new() -> Self {
        Self {
            // config: Config::new(), // i found out egui has its own thing for saving stuff, so default is fine
            ..Self::default()
        }
    }

    /// Opens a directory and mutates the pathbuf to contain it
    fn open_directory(output_path: &mut PathBuf) {
        let path = AutocropApp::path_or_desktop(output_path);
        let path = FileDialog::new()
            .set_location(&path)
            .show_open_single_dir()
            .unwrap();

        let path = match path {
            Some(path) => path,
            None => return,
        };

        *output_path = path;
    }

    /// If the path doesnt exist, default to desktop, and if that somehow fails just open the pathbuf default location.
    fn path_or_desktop(path: &PathBuf) -> PathBuf {
        if path.exists() {
            path.clone()
        } else {
            PathBuf::from_str("~/Desktop").unwrap_or(PathBuf::default())
        }
    }

    /// Function that opens and loads images and textures into the program.
    fn open_files(
        alloc: &mut dyn epi::TextureAllocator,
        tex_manager: &mut TextureManager,
        sender: Sender<DynamicImage>,
        input_path: &mut PathBuf,
    ) {
        let path = AutocropApp::path_or_desktop(input_path);
        let paths = FileDialog::new()
            .set_location(&path)
            .add_filter("PNG Image", &["png"])
            .show_open_multiple_file()
            .unwrap();

        // alert if they pick one image, silently return if they pick none (window closed?)
        match paths.len() {
            0 => return,
            1 => {
                MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("Alert")
                    .set_text("At minimum two images must be selected.")
                    .show_alert()
                    .unwrap();
            }
            2..=10000 => {
                // check if images are the same resolution, if not return
                let mut iter = paths.iter();
                // todo: give user feedback when the readers fail..?
                let (width, height) = image::io::Reader::open(iter.next().unwrap())
                    .unwrap()
                    .into_dimensions()
                    .unwrap();
                for path in iter {
                    if (width, height)
                        != image::io::Reader::open(path)
                            .unwrap()
                            .into_dimensions()
                            .unwrap()
                    {
                        MessageDialog::new()
                            .set_type(MessageType::Info)
                            .set_title("Alert")
                            .set_text("Images must be the same resolution.")
                            .show_alert()
                            .unwrap();
                        return;
                    }
                }

                // actually load the textures
                *input_path = paths[0].clone();
                tex_manager.input_paths = paths;
                tex_manager.reload_textures(alloc, sender);
            }
            _ => {
                MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("10k+ images")
                    .set_text("You cannot select more than 10000 images.")
                    .show_alert()
                    .unwrap();
            }
        };
    }
}

impl epi::App for AutocropApp {
    fn name(&self) -> &str {
        "Autocrop"
    }

    // Called by the framework to load old app state (if any).
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        // todo? i'm not sure if its worth having a option around config so that it's only set once
        // if storage & config exist, set app's config to it. otherwise it should be the default value.
        if let Some(storage) = storage {
            if let Some(config) = epi::get_value(storage, "config") {
                self.config = config;
            }
        }
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, "config", &self.config);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a [`egui::SidePanel`], [`egui::TopBottomPanel`], [`egui::CentralPanel`], [`egui::Window`] or [`egui::Area`].
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        // there are borrow issues when using app's properties inside |ui| {}
        // so this seems to be the workabout to use values of the app struct
        let Self {
            tex_manager,
            reciever,
            config,
        } = self;

        // Everything takes place on a scrollable central panel
        // this is mostly for legacy reasons, but, also because it is technically possible
        // for the output directory text to expand the content past the minimum window height
        egui::CentralPanel::default().show(ctx, |ui| {
            // scroll area's do not disable scrolling on ui disable, despite the scroll bar being useless, 
            // so disable scrolling manually on each one when ui isnt enabled
            // also scrollbar adjust for the bottom panel
            egui::ScrollArea::from_max_height(ui.clip_rect().height() - SCROLLBAR_ADJUST)
            .enable_scrolling(ui.enabled()).show(ui, |ui| {
                // Disable all widgets while loading new images in.
                if reciever.is_some() {
                    let cont = tex_manager.update_textures(frame.tex_allocator(), reciever.as_mut().unwrap());
                    if cont {
                        ctx.request_repaint(); // while loading textures request repaint?
                        ui.set_enabled(false);
                    } else {
                        *reciever = None;
                    }
                }

                // title is centered
                ui.vertical_centered(|ui| {
                    ui.heading("rui's super cool auto crop tool");
                    ui.add_space(15.0);
                });

                // combo box and checkbox.
                // centering is stupid, just stop thinking about it
                ui.horizontal(|ui| {
                    ui.add_space(28.0);
                    egui::ComboBox::from_label("Crop Type")
                    .selected_text(config.crop_type.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.crop_type, CropType::Rectangle, CropType::Rectangle.name())
                            .on_hover_text(CropType::Rectangle.tooltip());
                        ui.selectable_value(&mut config.crop_type, CropType::Exact, CropType::Exact.name())
                            .on_hover_text(CropType::Exact.tooltip());
                    });
                    ui.add_space(ui.available_width()-118.0);
                    ui.checkbox(&mut config.resize_output, "resize output")
                    .on_hover_text("When false, cropped out space\nis replaced with empty pixels.");
                });

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

                // output directory & browse button on the left and right done through columnss
                ui.columns(2, |columns| {
                    columns[0].heading("output directory");
                    columns[1].with_layout(egui::Layout::top_down(Align::Max), |ui| {
                        if ui.button("Browse").clicked() {
                            AutocropApp::open_directory(&mut config.output_path);
                        }
                    });
                });
                ui.add_space(5.0);

                // scrollable output path in the case that you somehow feed it a path longer than ~4 lines
                let scroll_area = egui::ScrollArea::from_max_height(60.0).enable_scrolling(ui.enabled()).id_source("output scroll");
                scroll_area.show(ui, |ui| {
                    ui.add_sized(
                        [ui.available_width(), 25.0],
                        egui::TextEdit::multiline(&mut config.output_path.to_string_lossy().to_string())
                            .desired_rows(1)
                            .enabled(false)
                    );
                });

                // file names
                // todo: fix this stuff up
                // yay magic width numbers because egui is pain
                let combobox_width = 120.0;
                let textbox_width = 200.0;
                let min_clip_width = 687.0;
                let padding = 16.5; // probably
                let center_padding = (ui.clip_rect().width()/2.0-(combobox_width+textbox_width+padding)/2.0-3.0)
                                    .at_least(0.0);
                ui.horizontal(|ui|{
                    if ui.clip_rect().width() < min_clip_width {
                        ui.add_space(center_padding);
                    }
                    ui.style_mut().override_text_style = Some(TextStyle::Monospace);

                    egui::ComboBox::from_id_source("bg_name")
                    .selected_text(config.bg_name.name_type.name())
                    .width(combobox_width)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.bg_name.name_type, NameType::Original, NameType::Original.name())
                            .on_hover_text(NameType::Original.tooltip());
                        ui.selectable_value(&mut config.bg_name.name_type, NameType::Custom, NameType::Custom.name())
                            .on_hover_text(NameType::Custom.tooltip());
                        ui.label("test?");
                    });
                    let textedit = egui::TextEdit::singleline(&mut config.bg_name.name)
                                    .enabled(config.bg_name.name_type == NameType::Custom);
                    ui.add_sized([textbox_width, 20.0], textedit)
                    .on_hover_text(format!("{}.png", config.bg_name.name));
                    // space between the two. yep, more magic width numbers.
                    ui.add_space(ui.available_width()-textbox_width-combobox_width-padding);
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
                            ui.selectable_value(&mut config.file_name.name_type, NameType::Original, NameType::Original.name())
                                .on_hover_text(NameType::Original.tooltip());
                            ui.selectable_value(&mut config.file_name.name_type, NameType::Custom, NameType::Custom.name())
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
                            ui.selectable_value(&mut config.file_name.name_type, NameType::Original, NameType::Original.name())
                                .on_hover_text(NameType::Original.tooltip());
                            ui.selectable_value(&mut config.file_name.name_type, NameType::Custom, NameType::Custom.name())
                                .on_hover_text(NameType::Custom.tooltip());
                        });
                        let textedit = egui::TextEdit::singleline(&mut config.file_name.name)
                                        .enabled(config.file_name.name_type == NameType::Custom);
                        ui.add_sized([textbox_width, 20.0], textedit)
                            .on_hover_text(format!("{}123.png", config.file_name.name));
                    });
                }
                ui.add_space(20.0);

                // files selected and its browse button
                ui.columns(2, |columns| {
                    // if loading images change the text to say how many are loading
                    let string = {
                        let path_len = tex_manager.input_paths.len();
                        let image_len = tex_manager.images.len();

                        if path_len != image_len {
                            format!("{}/{} files loaded", image_len, path_len)
                        } else {
                            format!("{} files selected", path_len)
                        }
                    };

                    columns[0].heading(string);
                    columns[1].with_layout(egui::Layout::top_down(Align::Max), |ui| {
                        if ui.button("Browse").clicked() {
                            let (tx, rx): (Sender<DynamicImage>, Receiver<DynamicImage>) = mpsc::channel();
                            *reciever = Some(rx);
                            AutocropApp::open_files(frame.tex_allocator(), tex_manager, tx, &mut config.input_path);
                        }
                    });
                });
                ui.add_space(5.0);

                // image previewer, this is the most complicated part
                if tex_manager.textures.len() == 0 {
                    // small placeholder text until images are avilable
                    ui.vertical_centered(|ui| {
                        ui.label("(images will appear here)");
                    });
                } else {
                    // "add" is the space needed between here and the top of the crop bottom
                    let add = (ui.clip_rect().size().y - ui.min_size().y - LOWER_PANEL_HEIGHT - DEFAULT_PREVIEW_HEIGHT)
                        .at_least(0.0);
                    egui::ScrollArea::from_max_height(DEFAULT_PREVIEW_HEIGHT+add)
                    .enable_scrolling(ui.enabled()).show(ui, |ui| {
                        // extra columns when theres enough width
                        let mut columns = (ui.available_width() / tex_manager.textures[0].width as f32).floor() as usize;
                        if tex_manager.textures.len() < columns {
                            columns = tex_manager.textures.len();
                        }
                        columns = columns.at_least(3);

                        egui::Grid::new("table")
                            // max and min the same, always a third of the window, min required for height limit
                            // no min row height so that they stack right on top of eachother when the window is minimized to small amounts
                            .max_col_width(ui.available_width()/columns as f32-4.0)
                            .min_col_width(ui.available_width()/columns as f32-4.0)
                            .spacing(egui::Vec2::new(5.0,5.0))
                            .show(ui, |ui| {
                                let mut counter = 0;
                                for tex in &tex_manager.textures {
                                    if counter == columns {
                                        counter = 0;
                                        ui.end_row();
                                    }
                                    counter+=1;

                                    // stuff to make sure it fits nice n snug but isnt too tall or wide
                                    let ratio: f32 = tex.height as f32 / tex.width as f32;
                                    let predicted_height = ui.available_width() * ratio;
                                    if predicted_height > PREVIEW_IMAGE_HEIGHT {
                                        ui.vertical_centered(|ui| {
                                            ui.image(tex.id, egui::Vec2::new(PREVIEW_IMAGE_HEIGHT/ratio, PREVIEW_IMAGE_HEIGHT));
                                        });
                                    } else {
                                        // fit to available space only if the width is larger than the available space, otherwise just do normal texture width
                                        if ui.available_width() < tex.width as f32 {
                                            ui.vertical_centered(|ui| {
                                                ui.image(tex.id, egui::Vec2::new(ui.available_width(), predicted_height));
                                            });
                                        } else {
                                            ui.vertical_centered(|ui| {
                                                ui.image(tex.id, egui::Vec2::new(tex.width as f32, tex.height as f32));

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

                // lower panel adjustment space, add space to move it to the bottom of the window (when its large enough)
                let mut add = ui.clip_rect().size().y - ui.min_size().y - LOWER_PANEL_HEIGHT;
                add = add.at_least(0.0);
                ui.add_space(10.0+add);

                // crop button, justified fills left and right making the button big and shiny
                // disabled until conditions are met
                let crop_enabled =  !(tex_manager.input_paths.len() == 0 || config.output_path.as_os_str().is_empty());
                ui.vertical_centered_justified(|ui| {
                    let button =  egui::widgets::Button::new("Crop").enabled(crop_enabled);

                    if ui.add_sized([0.0, 50.0], button)
                        .on_disabled_hover_text("Missing output path or input images")
                        .clicked() {
                            // todo: handle this hah
                            crate::crop(&mut tex_manager.images, config).unwrap();
                        }
                });

            });
        });

        // bottom panel, displays debug build text (if in debug) + version number
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.columns(2, |columns| {
                egui::warn_if_debug_build(&mut columns[0]);
                columns[1].with_layout(egui::Layout::top_down(Align::Max), |ui| {
                    ui.label(format!("{}", VERSION));
                });
            });
        });
    }
}
