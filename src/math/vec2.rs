use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

pub const EPS: f32 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
impl Vector2 {
    pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };
    pub const ONE: Vector2 = Vector2 { x: 1.0, y: 1.0 };
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }
    pub fn is_zero(self) -> bool {
        self.x.abs() < EPS && self.y.abs() < EPS
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < EPS {
            Self::ZERO
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }
    pub fn rotate(self, angle_radians: f32) -> Self {
        let (c, s) = (angle_radians.cos(), angle_radians.sin());
        Self::new(c * self.x - s * self.y, s * self.x + c * self.y)
    }
    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    pub fn scale(self, sx: f32, sy: f32) -> Self {
        Self::new(self.x * sx, self.y * sy)
    }
    pub fn clamp_magnitude(v: Self, max: f32) -> Self {
        let len_sq = v.length_sq();
        if len_sq <= max * max {
            v
        } else {
            v * (max / len_sq.sqrt())
        }
    }
    pub fn velocity(direction: Self, speed: f32) -> Vector2 {
        if Vector2::is_zero(direction) {
            Vector2::ZERO
        } else {
            direction.normalize() * speed
        }
    }

    pub fn move_towards(pos: &mut Vector2, target: Vector2, speed: f32, delta: f32) {
        let to_target = target - *pos;
        let dist_sq = to_target.length_sq();

        if dist_sq < EPS {
            *pos = target;
            return;
        }
        let dist = dist_sq.sqrt();
        let max_step = speed * delta;
        if dist <= max_step {
            *pos = target;
        } else {
            let dir = to_target / dist;
            *pos += dir * max_step;
        }
    }
    pub fn from_direction(direction: Self, speed: f32) -> Self {
        if direction.is_zero() {
            Self::ZERO
        } else {
            direction.normalize() * speed
        }
    }
}

impl Default for Vector2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}
impl Mul<f32> for Vector2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}
impl Mul<Vector2> for Vector2 {
    type Output = Self;
    fn mul(self, rhs: Vector2) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}
impl Div<f32> for Vector2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}
impl AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl AddAssign<f32> for Vector2 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
    }
}
impl SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, rhs: Vector2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl MulAssign<Vector2> for Vector2 {
    fn mul_assign(&mut self, rhs: Vector2) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}
impl MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}
