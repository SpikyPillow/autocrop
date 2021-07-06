use std::{cmp::Ordering, path::PathBuf, sync::mpsc::{self, Receiver, Sender}, thread};

use eframe::{egui::{Color32, TextureId}, epi};
use image::{DynamicImage, GenericImageView, imageops};

use crate::app::{PREVEW_IMAGE_LIMIT, PREVIEW_IMAGE_HEIGHT, PREVIEW_IMAGE_WIDTH};

#[derive(Default)]
/// Texture manager holds onto the images and image paths, and is also responsible for their loading.
pub struct TextureManager {
    pub textures: Vec<Texture>,
    pub images: Vec<DynamicImage>,
    pub input_paths: Vec<PathBuf>,
}

impl TextureManager {
    /// Called only explicitly when the texture lists have changed, is in charge of spawning the new thread to open images.
    pub fn reload_textures (&mut self, alloc: &mut dyn epi::TextureAllocator, sender: Sender<DynamicImage>) {
        // make sure to remove the textures first
        for tex in &self.textures {
            alloc.free(tex.id);
        }
        self.textures = vec![];
        self.images = vec![];

        // load the textures on a seperate thread because blocking
        let paths = self.input_paths.to_owned();
        thread::spawn(move || {
            for path in paths.to_owned() {
                sender.send(image::open(path).unwrap()).unwrap();
            }
        });
    }
    
    /// Called explicitly when looking for new textures from the other thread to add.
    /// Returns true if the transmitter is still alive.
    pub fn update_textures (&mut self, alloc: &mut dyn epi::TextureAllocator, rx: &mut Receiver<DynamicImage>) -> bool {
        // loads 3 images of backup at once per frame, or if theres none it continues the frame as normal
        for (num, image) in rx.try_iter().enumerate() {
            self.load_texture(alloc, image);
            if num == 1 { break; }
        }
        match rx.try_recv() {
            Ok(image) => { 
                self.load_texture(alloc, image);

                true
            },
            Err(mpsc::TryRecvError::Disconnected) => false,
            _ => true
        }
    }

    /// Load an individual texture into memory. used by update_textures for each individual texture.
    /// This is where resizing happens
    fn load_texture(&mut self, alloc: &mut dyn epi::TextureAllocator, image: DynamicImage) {
        let (width, height);
        // hard limit of previews for ram
        if self.textures.len() <= PREVEW_IMAGE_LIMIT { 
            let id = {
                let img = TextureManager::preview_resize(&image)
                    .into_rgba8();
                width = img.width() as usize;
                height = img.height() as usize;
                if width > 300 || height > 250 {
                    dbg!(width, height);
                }
                let pixels = img.pixels().map(|pixel| {
                    Color32::from_rgba_premultiplied(pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3])
                });
                let pixels: Vec<_> = pixels.collect();
    
                alloc.alloc_srgba_premultiplied((width, height), &pixels)
            };
    
            let tex = Texture::new(id, width, height);
            self.textures.push(tex);
        }
        self.images.push(image);
    }

    /// Creates new image from the first, resized at preview maxes
    fn preview_resize(image: &DynamicImage) -> DynamicImage {
        match image.width().cmp(&image.height()) {
            Ordering::Less => {
                // width less than height, resize off height
                image.resize(u32::MAX, PREVIEW_IMAGE_HEIGHT as u32, imageops::Nearest)
            },
            Ordering::Greater | Ordering::Equal => {
                // height less than width, resize off width
                image.resize(PREVIEW_IMAGE_WIDTH as u32, u32::MAX, imageops::Nearest)
            },
        }
    }
}

/// Holds the texture id (used to render and free stuff on gui) and width/height for convenience
pub struct Texture {
    pub id: TextureId,
    pub width: usize,
    pub height: usize,
}

impl Texture {
    pub fn new (id: TextureId, width: usize, height: usize) -> Self {
        Texture {
            id,
            width,
            height,
        }
    }
}