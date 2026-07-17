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
    pub remainder: Vector2,
}
impl Component for Body {
    /*
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, _delta: f32) {
        match self.body_type {
            BodyType::Character => {
                let mut snapped = false;

                let flags =
                    ctx.move_and_slide(base.id, &mut base.transform.position, &mut self.velocity);

                if flags.on_floor && self.velocity.y >= 0.0 {
                    if let Some(snap) = ctx.snap_to_floor(base.id, self.floor_snap_length) {
                        base.transform.position.y += snap as f32;
                        ctx.translate_my_colliders(base.id, Vector2i::new(0, snap));
                        snapped = true;

                        if self.velocity.y > 0.0 {
                            self.velocity.y = 0.0;
                        }
                    }
                }

                self.on_floor = snapped || flags.on_floor;
                self.on_wall = flags.on_wall;
                self.on_ceiling = flags.on_ceiling;
            }
            BodyType::Static => {}
        }
    }
    */
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

    pub fn move_and_slide(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if matches!(self.body_type, BodyType::Static) {
            return;
        }

        self.on_floor = false;
        self.on_wall = false;
        self.on_ceiling = false;

        let frame_movement = self.velocity * delta;
        let max_step = 4.0;
        let distance = frame_movement.length();
        let steps = (distance / max_step).ceil() as i32;

        if steps > 0 {
            let mut step_movement = frame_movement / (steps as f32);

            for _ in 0..steps {
                self.remainder += step_movement;

                let step_x = self.remainder.x.trunc();
                let step_y = self.remainder.y.trunc();

                self.remainder.x -= step_x as f32;
                self.remainder.y -= step_y as f32;

                let mut fake_floor_check = false;
                if step_y == 0.0 && self.velocity.y >= 0.0 {
                    fake_floor_check = true;
                }

                if step_x == 0.0 && step_y == 0.0 {
                    continue;
                }

                let mut pos_i = base.transform.position;
                let mut vel_i = Vector2::new(step_x, step_y);
                let old_pos_i = pos_i;

                let flags = ctx.move_and_slide(base.id, &mut pos_i, &mut vel_i);

                base.transform.position.x += (pos_i.x - old_pos_i.x) as f32;
                base.transform.position.y += (pos_i.y - old_pos_i.y) as f32;

                if flags.on_wall {
                    self.on_wall = true;
                    self.velocity.x = 0.0;
                    step_movement.x = 0.0;
                    self.remainder.x = 0.0;
                }

                if flags.on_floor {
                    self.on_floor = true;
                    self.velocity.y = 0.0;
                    step_movement.y = 0.0;
                    self.remainder.y = 0.0;
                } else if fake_floor_check {
                    base.transform.position.y -= 1.0;
                    ctx.translate_my_colliders(base.id, Vector2i::new(0, -1));
                }

                if flags.on_ceiling {
                    self.on_ceiling = true;
                    self.velocity.y = 0.0;
                    step_movement.y = 0.0;
                    self.remainder.y = 0.0;
                }
            }
        }

        if self.on_floor && self.velocity.y >= 0.0 {
            if let Some(snap) = ctx.snap_to_floor(base.id, self.floor_snap_length) {
                base.transform.position.y += snap as f32;
                ctx.translate_my_colliders(base.id, Vector2i::new(0, snap));
                self.on_floor = true;

                if self.velocity.y > 0.0 {
                    self.velocity.y = 0.0;
                }
            }
        }
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
            remainder: Vector2::ZERO,
        }
    }
}
