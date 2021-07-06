#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod texture;
mod config;

pub use app::AutocropApp;
use image::ImageBuffer;
use image::Rgba;

use std::{error::Error};
use config::Config;

use image::ImageFormat;
use image::GenericImageView;
use image::DynamicImage;

// #[derive(Clone, Copy, Default, PartialEq)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default, Debug, PartialEq)]
struct Pos2 {
    x: u32,
    y: u32,
}

impl Pos2 {
    fn new(x:u32, y:u32) -> Self {
        Self {x,y}
    }
}

#[derive(Debug)]
struct RectangleRange {
    min: Pos2,
    max: Pos2,
}

impl RectangleRange {
    fn new() -> Self {
        Self {
            min: Pos2::new(u32::MAX,u32::MAX),
            max: Pos2::new(0,0),
        }
    }

    fn width(&self) -> u32 {
        self.max.x - self.min.x
    }

    fn height(&self) -> u32 {
        self.max.y - self.min.y
    }

    fn correct(&mut self, x:u32, y:u32) -> bool {
        let mut result = false;
        if x < self.min.x { 
            self.min.x = x;
            result = true;
        }
        if y < self.min.y { 
            self.min.y = y;
            result = true;
        }

        if x > self.max.x { 
            self.max.x = x;
            result = true;
        }
        if y > self.max.y { 
            self.max.y = y; 
            result = true;
        }

        result
    }

    fn contains(&self, x:u32, y:u32) -> bool {
        x >= self.min.x && x <= self.max.x && y >= self.min.y && y <= self.max.y
    }
}

pub fn crop(images: &mut Vec<DynamicImage>, config: &Config) -> Result<(), Box<dyn Error>> {
    println!("starting crop: figuring out range of area to work with");
    let bg = &images[0];
    let mut range = RectangleRange::new();
    
    // get range of crop area
    for (x, y, bg_px) in bg.pixels() {
        // we're comparing these images to the background, so skip background
        for image in images.iter().skip(1) {
            let px = image.get_pixel(x, y);
            // f64 here because i want to be a bit more precise with difference
            if difference(bg_px, px) > config.leniency as f64/100.0  {
                // if the range is corrected, that means a difference has been found, 
                // no need to cycle through the rest of the images, break here
                if range.correct(x,y) {
                    break;
                }
            }
        }
    }
    dbg!(&range);

    // if exact croptype, figure out the exact different pixels per image now
    
    // first vec is for images, second is for groups of everydifferent pixel
    // does not contain the background image, since everything is compared against it
    let mut different_pixels: Vec<Vec<Pos2>> = vec![];
    
    if config.crop_type == CropType::Exact {
        // populate first vector per image
        for _ in images.iter().skip(1) {
            different_pixels.push(vec![]);
        }

        for (x, y, bg_px) in bg.pixels() {
            if range.contains(x, y) {
                // we're comparing these images to the background, so skip background
                for (i,image) in images.iter().skip(1).enumerate() {
                    let px = image.get_pixel(x, y);
                    if difference(bg_px, px) > config.leniency as f64/100.0  {
                        different_pixels[i].push(Pos2::new(x, y));
                    }
                }

            }
        }
    }

    for (i, image) in images.iter_mut().enumerate() {
        // if first image (bg), return itself
        let img = if i == 0 {
            println!("cropping background image");
            image.clone()
        // non bg images
        } else {
            println!("cropping image {}...", i);
            match config.crop_type {
                CropType::Rectangle => {
                    if config.resize_output == false {
                        DynamicImage::ImageRgba8(ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
                            if range.contains(x, y) {
                                image.get_pixel(x, y)
                            } else {
                                Rgba([0,0,0,0])
                            }
                        }))
                    } else {
                        image.crop(range.min.x, range.min.y, range.width(), range.height())
                    }
                },
                CropType::Exact => {
                    let mut new = ImageBuffer::new(image.width() , image.height());
                    // for every different position on the image, copy it over
                    for Pos2{x,y} in different_pixels[i-1].iter() {
                        new.put_pixel(*x, *y, image.get_pixel(*x, *y));
                    }
                    
                    let mut new = DynamicImage::ImageRgba8(new);
                    
                    if config.resize_output {
                        new.crop(range.min.x, range.min.y, range.width(), range.height())
                    } else {
                        new
                    }
                },
            }
            
        };

        let mut path = config.output_path.clone();
        path.push(format!("{}.png",i));
        println!("saving image {}...", i);
        img.save_with_format(path, ImageFormat::Png)?;
    }

    println!("done!");
    Ok(())
}

/// Returns a 0-1.0 value of how "close" the pixels are to eachother
fn difference(px1: Rgba<u8>, px2: Rgba<u8>) -> f64 {
    let difference = ((px1[0] as i32 - px2[0] as i32).pow(2)) +
                     ((px1[1] as i32 - px2[1] as i32).pow(2)) + 
                     ((px1[2] as i32 - px2[2] as i32).pow(2));
    
    difference as f64 / 195075.0 
}

// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

use crate::config::CropType;

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = AutocropApp::default();
    eframe::start_web(canvas_id, Box::new(app))
}
