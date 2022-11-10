use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};

pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Vec2 {x, y}
    }

    pub fn mag_squared(self) -> f64 {
        self.x*self.x + self.y*self.y
    }

    pub fn mag(self) -> f64 {
        self.mag_squared().sqrt()
    }

    pub fn normalize(self) -> Vec2 {
        self / self.mag()
    }

    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;

    fn mul(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(self, other: f64) -> Vec2 {
        Vec2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f64> for Vec2 {
    type Output = Vec2;

    fn div(self, other: f64) -> Vec2 {
        Vec2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, other: f64) {
        *self = Self {
            x: self.x * other,
            y: self.y * other,
        };
    }
}

impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, other: f64) {
        *self = Self {
            x: self.x - other,
            y: self.y - other,
        };
    }
}