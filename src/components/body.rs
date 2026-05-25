use crate::{Base, Component, EngineApi, Vector2};

pub struct Body2D {
    pub velocity: Vector2,
}
impl Component for Body2D {}

impl Body2D {
    pub fn move_and_slide(&mut self, ctx: &mut impl EngineApi, base: &mut Base) {
        ctx.move_and_slide(base.id, &mut base.transform.position, &mut self.velocity);
    }
}
