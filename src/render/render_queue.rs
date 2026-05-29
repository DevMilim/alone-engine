use crate::{Anchor, Color, DrawCommand, Rect, RenderApi, Vector2};

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
        z_index: u8,
    ) {
        self.queue[z_index as usize].push(DrawCommand::Sprite {
            position,
            image: texture,
            anchor,
            source,
        })
    }

    fn camera_mut(&mut self) -> &mut crate::Vector2 {
        self.camera
    }
}
