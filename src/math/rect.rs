use crate::math::Vector2;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
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
    pub fn intersects(&self, other: &Self) -> bool {
        self.min_x() < other.max_x()
            && self.max_x() > other.min_x()
            && self.min_y() < other.max_y()
            && self.max_y() > other.min_y()
    }

    pub const fn from_center(center: Vector2, size: Vector2) -> Self {
        let x = round_to_i32(center.x - size.x / 2.0);
        let y = round_to_i32(center.y - size.y / 2.0);

        Self::new(x, y, round_to_i32(size.x), round_to_i32(size.y))
    }
}
const fn round_to_i32(value: f32) -> i32 {
    if value >= 0.0 {
        (value + 0.5) as i32
    } else {
        (value - 0.5) as i32
    }
}
