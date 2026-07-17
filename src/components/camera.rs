use crate::{
    core::{Base, Component, EngineApi, RenderApi},
    math::Vector2,
    render::{LOGICAL_HEIGHT, LOGICAL_WIDTH},
};

pub struct Camera {
    pub position: Vector2,
    pub last_position: Vector2,
    pub active: bool,
    pub lerp_speed: f32,
    pub deadzone: Vector2,
    pub half: Vector2,
}

impl Camera {
    pub fn new(deadzone: Vector2, active: bool) -> Self {
        Self {
            active,
            lerp_speed: 10.0,
            deadzone,
            position: Vector2::ZERO,
            last_position: Vector2::ZERO,
            half: Vector2::ZERO,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vector2::ZERO, false)
    }
}

impl Component for Camera {
    fn start(&mut self, _ctx: &mut impl EngineApi, _base: &mut Base) {
        self.last_position = self.position;
    }
    fn late_update(&mut self, _ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if !self.active {
            return;
        }
        self.last_position = self.position;

        let mut target_pos = self.position;

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

        self.position.x += (target_pos.x - self.position.x) * t;
        self.position.y += (target_pos.y - self.position.y) * t;

        if (target_pos.x - self.position.x).abs() < 0.5 {
            self.position.x = target_pos.x;
        }
        if (target_pos.y - self.position.y).abs() < 0.5 {
            self.position.y = target_pos.y;
        }
    }
    fn draw(&mut self, _renderer: &mut impl RenderApi, _base: &Base, blending: f32) {
        if !self.active {
            return;
        }

        let pos = self.last_position.lerp(self.position, blending);

        let cam = _renderer.camera_mut();
        cam.x = pos.x;
        cam.y = pos.y;
    }
}
