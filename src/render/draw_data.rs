use crate::{Color, Handler, ImageAsset, Rect, Vector2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    Center,
    TopLeft,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawCommand {
    Sprite {
        position: Vector2,
        image: Handler<ImageAsset>,
        anchor: Anchor,
    },
    Rect {
        color: Color,
        rect: Rect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawData {
    pub rect: Rect,
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
            rect: Rect::new(0.0, 0.0, 0, 0),
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
