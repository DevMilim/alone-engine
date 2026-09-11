use std::{fs, io};

use fontdue::{Font, FontSettings};
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct FontAsset {
    pub font: Font,
}

impl FontAsset {
    pub fn new(path: &str) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        match Font::from_bytes(bytes, FontSettings::default()) {
            Ok(font) => Ok(Self { font }),
            Err(e) => Err(std::io::Error::other(e)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GlyphKey {
    pub font_id: usize,
    pub character: char,
    pub size_px: u32,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GlyphInfo {
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub width: u32,
    pub height: u32,
    pub advance: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
}

pub struct GlyphCache {
    pub atlas: Vec<u8>,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub row_height: u32,

    pub glyphs: FxHashMap<GlyphKey, GlyphInfo>,
}

impl GlyphCache {
    const INITIAL_SIZE: u32 = 256;
    const PADDING: u32 = 1;

    pub fn new() -> Self {
        Self {
            atlas: vec![0u8; (Self::INITIAL_SIZE * Self::INITIAL_SIZE) as usize],
            atlas_width: Self::INITIAL_SIZE,
            atlas_height: Self::INITIAL_SIZE,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            glyphs: FxHashMap::default(),
        }
    }
    pub fn atlas_slice(&self) -> (&[u8], u32) {
        (&self.atlas, self.atlas_width)
    }
    fn pack_into_atlas(&mut self, metrics: fontdue::Metrics, bitmap: &[u8]) -> GlyphInfo {
        let glyph_w = metrics.width as u32;
        let glyph_h = metrics.height as u32;

        if glyph_w == 0 || glyph_h == 0 {
            return GlyphInfo {
                atlas_x: 0,
                atlas_y: 0,
                width: 0,
                height: 0,
                advance: metrics.advance_width,
                bearing_x: metrics.xmin as f32,
                bearing_y: metrics.ymin as f32,
            };
        }

        if self.cursor_x + glyph_w + Self::PADDING > self.atlas_width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + Self::PADDING;
            self.row_height = 0;
        }

        if self.cursor_y + glyph_h > self.atlas_height {
            self.grow_atlas();
        }

        let (x, y) = (self.cursor_x, self.cursor_y);

        self.blit_glyph_bitmap(x, y, glyph_w, glyph_h, bitmap);

        self.cursor_x += glyph_w + Self::PADDING;
        self.row_height = self.row_height.max(glyph_h);

        GlyphInfo {
            atlas_x: x,
            atlas_y: y,
            width: glyph_w,
            height: glyph_h,
            advance: metrics.advance_width,
            bearing_x: metrics.xmin as f32,
            bearing_y: metrics.ymin as f32,
        }
    }

    fn blit_glyph_bitmap(&mut self, x: u32, y: u32, w: u32, h: u32, bitmap: &[u8]) {
        for row in 0..h {
            let src_start = (row * w) as usize;
            let src_row = &bitmap[src_start..src_start + w as usize];

            let dst_start = ((y + row) * self.atlas_width + x) as usize;
            self.atlas[dst_start..dst_start + w as usize].copy_from_slice(src_row);
        }
    }

    fn grow_atlas(&mut self) {
        let new_width = self.atlas_width * 2;
        let new_height = self.atlas_height * 2;
        let mut new_atlas = vec![0u8; (new_width * new_height) as usize];

        for row in 0..self.atlas_height {
            let old_start = (row * self.atlas_width) as usize;
            let old_row = &self.atlas[old_start..old_start + self.atlas_width as usize];

            let new_start = (row * new_width) as usize;
            new_atlas[new_start..new_start + self.atlas_width as usize].copy_from_slice(old_row);
        }

        self.atlas = new_atlas;
        self.atlas_width = new_width;
        self.atlas_height = new_height;

        self.glyphs.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
    }

    pub fn get(&self, key: &GlyphKey) -> Option<&GlyphInfo> {
        self.glyphs.get(key)
    }

    pub fn get_or_rasterize(
        &mut self,
        font: &fontdue::Font,
        font_id: usize,
        ch: char,
        size_px: u32,
    ) -> GlyphInfo {
        let key = GlyphKey {
            font_id,
            character: ch,
            size_px,
        };
        if let Some(info) = self.glyphs.get(&key) {
            return *info;
        }
        let (metrics, bitmap) = font.rasterize(ch, size_px as f32);
        let info = self.pack_into_atlas(metrics, &bitmap);
        self.glyphs.insert(key, info);
        info
    }
}
