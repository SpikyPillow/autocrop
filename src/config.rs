use std::{path::{PathBuf}};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)] // makes new values default when deserializing old state
/// Configuration struct, this is whats loaded and saved to keep state.
/// Also is what is passed down to the crop function.
pub struct Config {
    pub leniency: f32,
    pub resize_output: bool,
    pub crop_type: CropType,
    pub bg_name: FileName,
    pub file_name: FileName,

    pub output_path: PathBuf,
    pub input_path: PathBuf,
}

/// Using a "filename" struct so that i can keep the name stored in cache all the time
#[derive(Debug, Serialize, Deserialize)] 
pub struct FileName {
    pub name_type: NameType,
    pub name: String,
}

impl Default for FileName {
    fn default() -> Self {
        Self {
            name_type: NameType::default(),
            name: String::from("name"),
        }
    }
}

/// Enum that represents what the file output names should be.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NameType {
    Original,
    Custom
}


impl Default for NameType {
    fn default() -> Self {
        Self::Original
    }
}

impl NameType {
    pub fn name(&self) -> &str {
        match self {
            NameType::Original => "Original Name",
            NameType::Custom   => "Custom Name",
        }
    }

    pub fn tooltip(&self) -> &str {
        match self {
            NameType::Original => "Use the original input file's name.",
            NameType::Custom   => "Use a user supplied file name.",
        }
    }
}

// Selection variant enum for the type of crop that will be done
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CropType {
    Rectangle,
    Exact,
}

impl Default for CropType {
    fn default() -> Self {
        Self::Exact
    }
}

impl CropType {
    /// Full name of the enum variant
    pub fn name(&self) -> &str {
        match self {
            CropType::Rectangle => "Rectangle",
            CropType::Exact     => "Exact Difference",
        }
    }

    /// Tooltip for each enum variant
    pub fn tooltip(&self) -> &str {
        match self {
            CropType::Rectangle => "Crops out same space in\nall but the background image.",
            CropType::Exact     => "Cuts out exact pixels.\nSlower, but snaller. Recommended.",
        }
    }
}