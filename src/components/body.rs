use crate::{
    core::{Base, Component, EngineApi, GameObjectBase, IComponent},
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

impl Component for Body {}

impl Body {
    pub fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
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

    fn try_snap_to_floor(&mut self, ctx: &mut impl EngineApi, base: &mut Base) -> bool {
        if self.velocity.y < 0.0 {
            return false;
        }
        let Some(snap) = ctx.snap_to_floor(base.id, self.floor_snap_length) else {
            return false;
        };

        base.transform.position.y += snap as f32;
        ctx.translate_my_colliders(base.id, Vector2i::new(0, snap));
        self.velocity.y = self.velocity.y.min(0.0);
        true
    }

    pub fn move_and_slide(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if matches!(self.body_type, BodyType::Static) {
            return;
        }

        self.on_floor = false;
        self.on_wall = false;
        self.on_ceiling = false;

        let frame_movement = self.velocity * delta;
        let distance = frame_movement.length();
        let max_step = 4.0;
        let steps = (distance / max_step).ceil() as i32;

        if steps > 0 {
            let mut step_movement = frame_movement / steps as f32;

            for _ in 0..steps {
                self.remainder += step_movement;

                let step_x = self.remainder.x.trunc();
                let step_y = self.remainder.y.trunc();
                self.remainder.x -= step_x;
                self.remainder.y -= step_y;

                if step_x == 0.0 && step_y == 0.0 {
                    continue;
                }

                let old_pos = base.transform.position;
                let mut pos = old_pos;
                let mut vel = Vector2::new(step_x, step_y);

                let flags = ctx.move_and_slide(base.id, &mut pos, &mut vel);
                base.transform.position = old_pos + (pos - old_pos);

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
                }

                if flags.on_ceiling {
                    self.on_ceiling = true;
                    self.velocity.y = 0.0;
                    step_movement.y = 0.0;
                    self.remainder.y = 0.0;
                }
            }
        }

        self.on_floor |= self.try_snap_to_floor(ctx, base);
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

pub trait IBody: GameObjectBase + IComponent<Body> {
    fn velocity(&self) -> Vector2 {
        self.get_self().velocity
    }
    fn velocity_mut(&mut self) -> &mut Vector2 {
        &mut self.get_self_mut().velocity
    }
    fn is_on_floor(&self) -> bool {
        self.get_self().on_floor
    }
    fn is_on_wall(&self) -> bool {
        self.get_self().on_wall
    }
    fn is_on_ceiling(&self) -> bool {
        self.get_self().on_ceiling
    }
    fn move_and_slide(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let (body, base) = self.get_self_and_base_mut();
        body.move_and_slide(ctx, base, delta);
    }
}
