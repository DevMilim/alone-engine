use crate::{Base, Component, EngineApi, Vector2};

pub struct RiggidBody2D {
    pub velocity: Vector2,
}
impl Component for RiggidBody2D {
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        ctx.move_and_slide(
            base.id,
            &mut base.transform.position,
            &mut self.velocity,
            delta,
        );
    }
}
