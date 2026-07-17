use crate::math::Vector2i;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AABB {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl AABB {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn min_x(&self) -> i32 {
        self.x
    }

    pub fn max_x(&self) -> i32 {
        self.x + self.width
    }

    pub fn min_y(&self) -> i32 {
        self.y
    }

    pub fn max_y(&self) -> i32 {
        self.y + self.height
    }

    pub fn center(&self) -> Vector2i {
        Vector2i::new(self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn position(&self) -> Vector2i {
        Vector2i::new(self.x, self.y)
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min_x() < other.max_x()
            && self.max_x() > other.min_x()
            && self.min_y() < other.max_y()
            && self.max_y() > other.min_y()
    }

    pub fn get_overlap(&self, other: &AABB) -> Option<Vector2i> {
        if !self.intersects(other) {
            return None;
        }

        let overlap_x = if (self.min_x() + self.max_x()) < (other.min_x() + other.max_x()) {
            other.min_x() - self.max_x()
        } else {
            other.max_x() - self.min_x()
        };

        let overlap_y = if (self.min_y() + self.max_y()) < (other.min_y() + other.max_y()) {
            other.min_y() - self.max_y()
        } else {
            other.max_y() - self.min_y()
        };

        Some(if overlap_x.abs() < overlap_y.abs() {
            Vector2i::new(overlap_x, 0)
        } else {
            Vector2i::new(0, overlap_y)
        })
    }

    pub fn contains_point(&self, point: Vector2i) -> bool {
        point.x >= self.min_x()
            && point.x <= self.max_x()
            && point.y >= self.min_y()
            && point.y <= self.max_y()
    }
}
