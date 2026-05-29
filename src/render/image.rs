use std::path::Path;

use image::imageops::FilterType;

#[derive(Debug, Clone, PartialEq)]
pub struct ImageAsset {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
impl ImageAsset {
    pub fn load_from_file(path: &str) -> Self {
        let img = image::open(Path::new(path)).expect("Falha ao carregar textura");
        let rgba = img.to_rgba8();

        let dimensions = rgba.dimensions();

        Self {
            width: dimensions.0,
            height: dimensions.1,
            pixels: rgba.to_vec(),
        }
    }
    pub fn load_from_file_and_resize(path: &str, width: u32, height: u32) -> Self {
        let img = image::open(Path::new(path)).expect("Falha ao carregar textura");
        let resize = img.resize(width, height, FilterType::Nearest);
        let rgba = resize.to_rgba8();

        let dimensions = rgba.dimensions();

        Self {
            width: dimensions.0,
            height: dimensions.1,
            pixels: rgba.to_vec(),
        }
    }
}
