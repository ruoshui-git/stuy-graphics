use crate::RGB;
use std::ops;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec3(pub f64, pub f64, pub f64);

impl Vec3 {
    // pub fn _dot(a: &Self, b: &Self) -> f64 {
    //     a.0 * b.0 + a.1 * b.1 + a.2 * b.2
    // }

    // pub fn _cross(a: &Self, b: &Self) -> Self {
    //     Vec3(
    //         a.1 * b.2 - a.2 * b.1,
    //         a.2 * b.0 - a.0 * b.2,
    //         a.0 * b.1 - a.1 * b.0,
    //     )
    // }

    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
    pub fn z(&self) -> f64 {
        self.2
    }
}

impl Vec3 {
    pub const ZEROS: Self = Self(0., 0., 0.);
}

impl Vec3 {
    pub fn dot(&self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    /// Multiply two Vec3 field by field; .mul() is already used for dot product
    pub fn mul_across(&self, other: Self) -> Self {
        Vec3(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }

    pub fn cross(&self, other: Self) -> Self {
        Vec3(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }

    pub fn from_pt(point: (f64, f64, f64)) -> Self {
        Vec3(point.0, point.1, point.2)
    }

    pub fn norm(&self) -> Self {
        *self / self.mag()
    }

    pub fn mag(&self) -> f64 {
        (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
    }

    pub fn limit(&self, min: f64, max: f64) -> Self {
        Self(
            self.0.max(min).min(max),
            self.1.max(min).min(max),
            self.2.max(min).min(max),
        )
    }

    pub fn limit_max(&self, max: f64) -> Self {
        Self(self.0.min(max), self.1.min(max), self.2.min(max))
    }
}

impl ops::Mul for Vec3 {
    type Output = f64;
    fn mul(self, rhs: Self) -> Self::Output {
        self.dot(rhs)
    }
}

impl ops::Mul<i32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: i32) -> Self::Output {
        Vec3(
            self.0 * f64::from(rhs),
            self.1 * f64::from(rhs),
            self.2 * f64::from(rhs),
        )
    }
}

impl ops::Mul<Vec3> for i32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl From<&RGB> for Vec3 {
    fn from(color: &RGB) -> Self {
        Vec3(color.red as f64, color.green as f64, color.blue as f64)
    }
}

impl From<RGB> for Vec3 {
    fn from(color: RGB) -> Self {
        Vec3(color.red as f64, color.green as f64, color.blue as f64)
    }
}

impl From<Vec3> for RGB {
    fn from(v: Vec3) -> Self {
        Self {
            red: v.0.max(0.).min(255.) as u16,
            blue: v.2.max(0.).min(255.) as u16,
            green: v.1.max(0.).min(255.) as u16,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_limit() {
//         // this should be correct
//     }
// }
