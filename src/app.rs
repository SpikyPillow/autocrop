use std::{
    path::PathBuf,
    str::FromStr,
    sync::mpsc::{self, Receiver, Sender},
};

use eframe::{
    egui::{self, Align, NumExt},
    epi,
};
use image::DynamicImage;
use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::config::Config;
use crate::texture::TextureManager;
// auto crop user interface
use crate::ui as acui;

/// crate version, for the display on the bottom right
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
/// set max height the images in the previewer will be, this is what height resizes to if it is greater than width
pub const PREVIEW_IMAGE_HEIGHT: f32 = 250.0;
/// set max width the images in the previewer will be, this is what width resizes to if it is greater than height
pub const PREVIEW_IMAGE_WIDTH: f32 = 300.0;
/// max amount of preview images before program stops generating previews to save ram
pub const PREVEW_IMAGE_LIMIT: usize = 1000;
/// the minimum height for the previewer ui itself
pub const DEFAULT_PREVIEW_HEIGHT: f32 = 200.0;
/// the fake "lower panel" (crop button) height. hard coded because pain.
pub const LOWER_PANEL_HEIGHT: f32 = 75.0;
/// the lower panel height to adjust for to make the scrollbar not appear longer than it should.
pub const SCROLLBAR_ADJUST: f32 = 35.0;

#[derive(Default)]
pub struct AutocropApp {
    tex_manager: TextureManager,
    reciever: Option<Receiver<DynamicImage>>,
    config: Config,
}

impl AutocropApp {
    pub fn new() -> Self {
        Self::default()
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

        // Most the ui takes place on a scrollable central panel
        // this is mostly for legacy reasons, but, also because it is technically possible
        // for the output directory text to expand the content past the minimum window height
        egui::CentralPanel::default().show(ctx, |ui| {
            // scroll area's do not disable scrolling on ui disable, despite the scroll bar being useless,
            // so disable scrolling manually on each one when ui isnt enabled
            // also size adjustment for the bottom panel (otherwise the scrollbar takes up the whole space)
            egui::ScrollArea::from_max_height(ui.clip_rect().height() - SCROLLBAR_ADJUST)
                .enable_scrolling(ui.enabled())
                .show(ui, |ui| {
                    // Disable all widgets while loading new images in.
                    if reciever.is_some() {
                        let cont = tex_manager
                            .update_textures(frame.tex_allocator(), reciever.as_mut().unwrap());
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
                    });
                    ui.add_space(15.0);

                    // crop type
                    acui::croptype::draw_croptype_selector(ui, config);

                    // leniency slider
                    acui::leniency::draw_leniency_slider(ui, config);

                    // output directory & browse button on the left and right done through columnss
                    if acui::label_and_browse(ui, "output directory").clicked() {
                        println!("uh oh");
                        AutocropApp::open_directory(&mut config.output_path);
                    }
                    ui.add_space(5.0);

                    // scrollable output path in the case that you somehow feed it a path longer than ~4 lines
                    let scroll_area = egui::ScrollArea::from_max_height(60.0)
                        .enable_scrolling(ui.enabled())
                        .id_source("output scroll");
                    scroll_area.show(ui, |ui| {
                        ui.add_sized(
                            [ui.available_width(), 25.0],
                            egui::TextEdit::multiline(
                                &mut config.output_path.to_string_lossy().to_string(),
                            )
                            .desired_rows(1)
                            .enabled(false),
                        );
                    });

                    // file names
                    acui::filename::draw_filename_selector(ui, config);
                    ui.add_space(20.0);

                    // files selected and its browse button
                    // if loading images change the text to say how many are loaded
                    let label = {
                        let path_len = tex_manager.input_paths.len();
                        let image_len = tex_manager.images.len();

                        if path_len != image_len {
                            format!("{}/{} files loaded", image_len, path_len)
                        } else {
                            format!("{} files selected", path_len)
                        }
                    };

                    if acui::label_and_browse(ui, label).clicked() {
                        let (tx, rx): (Sender<DynamicImage>, Receiver<DynamicImage>) =
                            mpsc::channel();
                        *reciever = Some(rx);
                        AutocropApp::open_files(
                            frame.tex_allocator(),
                            tex_manager,
                            tx,
                            &mut config.input_path,
                        );
                    }
                    ui.add_space(5.0);

                    // image previewer
                    acui::previewer::draw_file_previewer(ui, tex_manager);

                    // lower panel adjustment space, add space to move it to the bottom of the window (when its large enough)
                    let mut add = ui.clip_rect().size().y - ui.min_size().y - LOWER_PANEL_HEIGHT;
                    add = add.at_least(0.0);
                    ui.add_space(10.0 + add);

                    // crop button
                    if acui::crop_button(ui, config, tex_manager).clicked() {
                        // todo: handle this?
                        crate::crop(&mut tex_manager.images, config).unwrap();
                    }
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
