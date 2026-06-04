use crate::{Color, Handler, ImageAsset, Rect, RenderApi, Vector2};

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
        flip_v: bool,
        flip_h: bool,
    },
    Rect {
        color: Color,
        rect: Rect,
    },
}

pub struct RenderQueue<'a> {
    pub queue: &'a mut [Vec<DrawCommand>; 6],
    pub camera: &'a mut Vector2,
}

impl<'a> RenderApi for RenderQueue<'a> {
    fn draw(&mut self, z_index: u8, command: DrawCommand) {
        self.queue[z_index as usize].push(command);
    }

    fn draw_rect(&mut self, rect: Rect, color: Color, z_index: u8) {
        self.queue[z_index as usize].push(DrawCommand::Rect { color, rect });
    }

    fn draw_sprite(
        &mut self,
        position: crate::Vector2,
        texture: crate::Handler<crate::ImageAsset>,
        anchor: Anchor,
        source: Option<Rect>,
        flip_v: bool,
        flip_h: bool,
        z_index: u8,
    ) {
        self.queue[z_index as usize].push(DrawCommand::Sprite {
            position,
            image: texture,
            anchor,
            source,
            flip_v,
            flip_h,
        })
    }

    fn camera_mut(&mut self) -> &mut crate::Vector2 {
        self.camera
    }
}
