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
        source: Option<Rect>,
    },
    Rect {
        color: Color,
        rect: Rect,
    },
}
