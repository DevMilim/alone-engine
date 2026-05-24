use crate::{Component, EngineApi, LOGICAL_HEIGHT, LOGICAL_WIDTH, RenderApi, Vector2};

pub struct Camera2D {
    pub position: Vector2,
    pub active: bool,
    pub lerp_speed: f32,
    pub deadzone: Vector2,
    pub half: Vector2,
}

impl Camera2D {
    pub fn new(deadzone: Vector2) -> Self {
        Self {
            active: true,
            lerp_speed: 10.0,
            deadzone,
            position: Vector2::ZERO,
            half: Vector2::ZERO,
        }
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new(Vector2::ZERO)
    }
}

impl Component for Camera2D {
    fn late_update(&mut self, ctx: &mut impl EngineApi, base: &mut crate::Base, delta: f32) {
        if !self.active {
            return;
        }
        let mut target_pos = *ctx.camera_mut();
        let cam = ctx.camera_mut();

        let center_x = base.transform.position.x + self.half.x;
        let center_y = base.transform.position.y + self.half.y;

        let desired_x = center_x - (LOGICAL_WIDTH as f32 / 2.0);
        let desired_y = center_y - (LOGICAL_HEIGHT as f32 / 2.0);

        let diff_x = desired_x - target_pos.x;
        if diff_x.abs() > self.deadzone.x {
            target_pos.x = if diff_x > 0.0 {
                desired_x - self.deadzone.x
            } else {
                desired_x + self.deadzone.x
            };
        }

        let diff_y = desired_y - target_pos.y;
        if diff_y.abs() > self.deadzone.y {
            target_pos.y = if diff_y > 0.0 {
                desired_y - self.deadzone.y
            } else {
                desired_y + self.deadzone.y
            };
        }
        let t = 1.0 - (-self.lerp_speed * delta).exp();

        cam.x += (target_pos.x - cam.x) * t;
        cam.y += (target_pos.y - cam.y) * t;

        if (target_pos.x - cam.x).abs() < 0.5 {
            cam.x = target_pos.x;
        }
        if (target_pos.y - cam.y).abs() < 0.5 {
            cam.y = target_pos.y;
        }

        self.position = *cam;
    }
}
