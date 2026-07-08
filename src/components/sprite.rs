use crate::{
    core::{Base, Component, EngineApi, RenderApi},
    math::Vector2,
    render::{Anchor, SpriteSrc},
};

pub struct Sprite {
    pub texture: SpriteSrc,
    pub offset: Vector2,
    pub anchor: Anchor,
    pub visible: bool,
    pub flip_v: bool,
    pub flip_h: bool,
    pub previous_position: Vector2,
}
impl Component for Sprite {
    fn start(&mut self, _ctx: &mut impl EngineApi, base: &mut Base) {
        self.previous_position = base.transform.global_position
    }
    fn update(&mut self, _ctx: &mut impl EngineApi, base: &mut Base, _delta: f32) {
        self.previous_position = base.transform.global_position;
    }
    fn draw(&mut self, renderer: &mut impl RenderApi, base: &Base, blending: f32) {
        if !self.visible {
            return;
        }
        let current_position = self
            .previous_position
            .lerp(base.transform.global_position, blending);
        renderer.draw_sprite(
            current_position + self.offset,
            self.texture.texture,
            self.anchor,
            self.texture.src,
            self.flip_v,
            self.flip_h,
            base.z_index,
        );
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture: SpriteSrc::default(),
            offset: Vector2::ZERO,
            anchor: Anchor::Center,
            visible: true,
            previous_position: Vector2::ZERO,
            flip_v: false,
            flip_h: false,
        }
    }
}
