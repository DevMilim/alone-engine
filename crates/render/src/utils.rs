use assets::ImageAsset;
use core::{Handler, Vector2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
}

#[derive(Debug)]
pub enum DrawCommandType {
    Sprite,
    Rect,
}

pub struct DrawCommand {
    pub cmd_type: DrawCommandType,
    pub material: DrawData,
}

#[derive(Debug, Clone)]
pub struct DrawData {
    pub pos: Vector2,
    pub size: Vector2,
    pub uv_min: Vector2,
    pub uv_max: Vector2,
    pub rotation: f32,
    pub color: Color,
    pub image: Handler<ImageAsset>,
    pub flip_h: bool,
    pub flip_v: bool,
}
impl Default for DrawData {
    fn default() -> Self {
        Self {
            pos: Vector2::ZERO,
            size: Vector2::new(32.0, 32.0),
            uv_min: Vector2::ZERO,
            uv_max: Vector2::ZERO,
            rotation: 0.0,
            color: Color::WHITE,
            image: Handler::new(0),
            flip_h: false,
            flip_v: false,
        }
    }
}
