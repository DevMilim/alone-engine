use crate::{Anchor, Component, Handler, ImageAsset, Vector2};

pub struct Sprite {
    texture: Handler<ImageAsset>,
    offset: Vector2,
    anchor: Anchor,
    visible: bool,
    previous_position: Vector2,
}
impl Component for Sprite {
    fn start(&mut self, _ctx: &mut impl crate::EngineApi, base: &mut crate::Base) {
        self.previous_position = base.transform.global_position
    }
    fn update(&mut self, _ctx: &mut impl crate::EngineApi, base: &mut crate::Base, _delta: f32) {
        self.previous_position = base.transform.global_position;
    }
    fn draw(&mut self, renderer: &mut impl crate::RenderApi, base: &crate::Base, blending: f32) {
        if !self.visible {
            return;
        }
        let current_position = self
            .previous_position
            .lerp(base.transform.global_position, blending);
        renderer.draw_sprite(
            current_position + self.offset,
            self.texture,
            self.anchor,
            base.z_index,
        );
    }
}
