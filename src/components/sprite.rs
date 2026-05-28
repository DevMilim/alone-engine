use crate::{Anchor, Component, Handler, ImageAsset, Vector2};

pub struct Sprite {
    texture: Handler<ImageAsset>,
    offset: Vector2,
    last_position: Vector2,
    anchor: Anchor,
}
impl Component for Sprite {
    fn start(&mut self, _ctx: &mut impl crate::EngineApi, base: &mut crate::Base) {
        self.last_position = base.transform.global_position
    }
    fn draw(&mut self, renderer: &mut impl crate::RenderApi, base: &crate::Base, blending: f32) {
        let current_position = self
            .last_position
            .lerp(base.transform.global_position, blending);
        renderer.draw_sprite(
            current_position + self.offset,
            self.texture,
            self.anchor,
            base.z_index,
        );
        self.last_position = base.transform.global_position;
    }
}
