use crate::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct Transform2D {
    pub position: Vector2,
    pub rotation: f32,
    pub scale: Vector2,

    pub global_rotation: f32,
    pub global_position: Vector2,
    pub global_scale: Vector2,
}

impl Transform2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vector2 { x, y },
            rotation: 0.0,
            scale: Vector2::ONE,
            global_position: Vector2 { x, y },
            global_rotation: 0.0,
            global_scale: Vector2::ONE,
        }
    }
    pub const EMPTY: Self = Self {
        position: Vector2::ZERO,
        rotation: 0.0,
        scale: Vector2::ONE,
        global_position: Vector2::ZERO,
        global_rotation: 0.0,
        global_scale: Vector2::ONE,
    };

    pub fn apply_parent(&mut self, parent: &Transform2D, inherit: bool) {
        if inherit {
            self.global_scale = self.scale * parent.scale;
            self.global_rotation = parent.global_rotation + self.rotation;

            let scaled_pos = self.position * parent.global_scale;

            let sin = parent.global_rotation.sin();
            let cos = parent.global_rotation.cos();

            let rotated_x = scaled_pos.x * cos - scaled_pos.y * sin;
            let rotated_y = scaled_pos.x * sin + scaled_pos.y * cos;
            self.global_position = Vector2 {
                x: parent.global_position.x + rotated_x,
                y: parent.global_position.y + rotated_y,
            }
        } else {
            self.global_position = self.position;
            self.global_rotation = self.rotation;
            self.global_scale = self.scale;
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::EMPTY
    }
}
