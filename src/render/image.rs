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
        let rgba = img.into_rgba8();

        let dimensions = rgba.dimensions();

        Self {
            width: dimensions.0,
            height: dimensions.1,
            pixels: rgba.into_raw(),
        }
    }
    pub fn load_from_file_and_resize(path: &str, width: u32, height: u32) -> Self {
        let img = image::open(Path::new(path)).expect("Falha ao carregar textura");
        let resize = img.resize(width, height, FilterType::Nearest);
        let rgba = resize.into_rgba8();

        let dimensions = rgba.dimensions();

        Self {
            width: dimensions.0,
            height: dimensions.1,
            pixels: rgba.into_raw(),
        }
    }

    pub fn generate_flip_v(&mut self) {
        let mut pixels = vec![0u8; self.pixels.len()];
        let row_bytes = (self.width * 4) as usize;

        for (src_row, dst_row) in self
            .pixels
            .chunks_exact(row_bytes)
            .zip(pixels.chunks_exact_mut(row_bytes).rev())
        {
            dst_row.copy_from_slice(src_row);
        }
    }
    pub fn generate_flip_h(&mut self) {
        let mut pixels = vec![0u8; self.pixels.len()];
        let row_bytes = (self.width * 4) as usize;

        for (src_row, dst_row) in self
            .pixels
            .chunks_exact(row_bytes)
            .zip(pixels.chunks_exact_mut(row_bytes))
        {
            for (src_pixel, dst_pixel) in src_row
                .chunks_exact(4)
                .zip(dst_row.chunks_exact_mut(4).rev())
            {
                dst_pixel.copy_from_slice(src_pixel);
            }
        }
    }
    pub fn generate_flip_hv(&mut self) {
        let mut pixels = vec![0u8; self.pixels.len()];

        for (src_pixel, dst_pixel) in self
            .pixels
            .chunks_exact(4)
            .zip(pixels.chunks_exact_mut(4).rev())
        {
            dst_pixel.copy_from_slice(src_pixel);
        }
    }
}
