use crate::Vector2;

#[derive(Clone, Copy)]
pub struct AABB {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl AABB {
    pub fn intersects(&self, other: &AABB) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
    pub fn get_overlap(&self, other: &AABB) -> Option<Vector2> {
        let center_a_x = self.x + self.width / 2.0;
        let center_a_y = self.y + self.height / 2.0;
        let center_b_x = other.x + other.width / 2.0;
        let center_b_y = other.y + other.height / 2.0;

        let distance_x = center_a_x - center_b_x;
        let distance_y = center_a_y - center_b_y;

        let min_distance_x = f32::midpoint(self.width, other.width);
        let min_distance_y = f32::midpoint(self.height, other.height);

        if distance_x.abs() < min_distance_x && distance_y.abs() < min_distance_y {
            let overlap_x = min_distance_x - distance_x.abs();
            let overlap_y = min_distance_y - distance_y.abs();

            if overlap_x < overlap_y {
                let sx = if distance_x > 0.0 { 1.0 } else { -1.0 };
                Some(Vector2 {
                    x: overlap_x * sx,
                    y: 0.0,
                })
            } else {
                let sy = if distance_y > 0.0 { 1.0 } else { -1.0 };
                Some(Vector2 {
                    x: 0.0,
                    y: overlap_y * sy,
                })
            }
        } else {
            None
        }
    }
}
