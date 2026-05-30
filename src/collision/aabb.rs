use crate::Vector2;

#[derive(Clone, Copy, Debug)]
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
        let left = other.x - (self.x + self.width);

        let right = (other.x + other.width) - self.x;

        let top = other.y - (self.y + self.height);
        let bottom = (other.y + other.height) - self.y;

        if left >= 0.0 || right <= 0.0 || top >= 0.0 || bottom <= 0.0 {
            return None;
        }
        let overlap_x = if left.abs() < right.abs() {
            left
        } else {
            right
        };

        let overlap_y = if top.abs() < bottom.abs() {
            top
        } else {
            bottom
        };

        if overlap_x.abs() < overlap_y.abs() {
            Some(Vector2::new(overlap_x, 0.0))
        } else {
            Some(Vector2::new(0.0, overlap_y))
        }
    }
}
