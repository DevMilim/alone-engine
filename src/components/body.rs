use crate::{
    core::{Base, Component, EngineApi},
    math::{Vector2, Vector2i},
};

pub enum BodyType {
    Static,
    Character,
}

pub struct Body {
    pub velocity: Vector2,
    pub on_floor: bool,
    pub on_wall: bool,
    pub on_ceiling: bool,
    pub body_type: BodyType,
    pub floor_snap_length: i32,
}
impl Component for Body {
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, _delta: f32) {
        println!(
            "ANTES pos={:?} vel={:?}",
            base.transform.position, self.velocity
        );
        match self.body_type {
            BodyType::Character => {
                let was_on_floor = self.on_floor;
                let mut snapped = false;

                if was_on_floor && self.velocity.y >= 0.0 {
                    if let Some(snap) = ctx.snap_to_floor(base.id, self.floor_snap_length) {
                        base.transform.position.y += snap as f32;
                        ctx.translate_my_colliders(base.id, Vector2i::new(0, snap));
                        snapped = true;

                        if self.velocity.y > 0.0 {
                            self.velocity.y = 0.0;
                        }
                    }
                }
                let flags =
                    ctx.move_and_slide(base.id, &mut base.transform.position, &mut self.velocity);

                self.on_floor = snapped || flags.on_floor;
                self.on_wall = flags.on_wall;
                self.on_ceiling = flags.on_ceiling;
            }
            BodyType::Static => {}
        }
    }
}

impl Body {
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

impl Default for Body {
    fn default() -> Self {
        Self {
            velocity: Vector2::ZERO,
            on_floor: false,
            on_wall: false,
            on_ceiling: false,
            body_type: BodyType::Static,
            floor_snap_length: 4,
        }
    }
}
