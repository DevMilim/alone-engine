use crate::{Base, Component, EngineApi, Vector2};

pub struct Body2D {
    pub velocity: Vector2,
    pub on_floor: bool,
    pub on_wall: bool,
    pub on_ceiling: bool,
}
impl Component for Body2D {}

impl Body2D {
    pub fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity
    }
    pub fn is_on_floor(&self) -> bool {
        self.on_floor
    }
    pub fn is_on_wall(&self) -> bool {
        self.on_wall
    }
    pub fn is_on_ceiling(&self) -> bool {
        self.on_ceiling
    }

    pub fn move_and_slide(&mut self, ctx: &mut impl EngineApi, base: &mut Base) {
        let flags = ctx.move_and_slide(base.id, &mut base.transform.position, &mut self.velocity);
        self.on_floor = flags.on_floor;
        self.on_wall = flags.on_wall;
        self.on_ceiling = flags.on_ceiling;
    }
}

impl Default for Body2D {
    fn default() -> Self {
        Self {
            velocity: Vector2::ZERO,
            on_floor: false,
            on_wall: false,
            on_ceiling: false,
        }
    }
}
