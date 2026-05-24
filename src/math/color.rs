#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub fn bytes(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
    pub fn blend_byte(dst: u8, src: u8, alpha: u8) -> u8 {
        let a = alpha as u32;
        let inv = 255 - a;
        let x = src as u32 * a + dst as u32 * inv;

        (((x + 128) * 257) >> 16) as u8
    }
    pub fn blend(dst: &[u8; 4], src: &[u8; 4]) -> [u8; 4] {
        let alpha = src[3];

        if alpha == 0 {
            return *dst;
        }
        if alpha == 255 {
            return *src;
        }

        let r = Self::blend_byte(dst[0], src[0], alpha);
        let g = Self::blend_byte(dst[1], src[1], alpha);
        let b = Self::blend_byte(dst[2], src[2], alpha);
        [r, g, b, 255]
    }
}
