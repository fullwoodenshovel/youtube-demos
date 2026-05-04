use std::{fmt::Debug, ops::{Add, Div, Mul, Neg, Sub}};

use macroquad::math::Vec2;

#[derive(Clone, Copy, PartialEq)]
pub struct Mat2 {
    data: [f32; 4]
}

pub const I: Mat2 = Mat2::new(1.0, 0.0, 0.0, 1.0);

impl Debug for Mat2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl Mul<Mat2> for Mat2 {
    type Output = Mat2;
    fn mul(self, rhs: Mat2) -> Self::Output {
        Mat2 {
            data: [
                self.data[0] * rhs.data[0] + self.data[1] * rhs.data[2], self.data[0] * rhs.data[1] + self.data[1] * rhs.data[3],
                self.data[2] * rhs.data[0] + self.data[3] * rhs.data[2], self.data[2] * rhs.data[1] + self.data[3] * rhs.data[3],
            ]
        }
    }
}

impl Mul<Mat2> for f32 {
    type Output = Mat2;
    fn mul(self, rhs: Mat2) -> Self::Output {
        Mat2 {
            data: [
                self * rhs.data[0], self * rhs.data[1],
                self * rhs.data[2], self * rhs.data[3]
            ]
        }
    }
}

impl Mul<f32> for Mat2 {
    type Output = Mat2;
    fn mul(self, rhs: f32) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vec2> for Mat2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2::new(rhs.x * self.data[0] + rhs.y * self.data[1], rhs.x * self.data[2] + rhs.y * self.data[3])
    }
}

impl Div<f32> for Mat2 {
    type Output = Mat2;
    fn div(self, rhs: f32) -> Self::Output {
        let mul = 1.0 / rhs;
        mul * self
    }
}

impl Add<Mat2> for Mat2 {
    type Output = Mat2;
    fn add(self, rhs: Mat2) -> Self::Output {
        Mat2 {
            data: [
                self.data[0] + rhs.data[0], self.data[1] + rhs.data[1],
                self.data[2] + rhs.data[2], self.data[3] + rhs.data[3],
            ]
        }
    }
}

impl Neg for Mat2 {
    type Output = Mat2;
    fn neg(self) -> Self::Output {
        Mat2 {
            data: [
                -self.data[0], -self.data[1],
                -self.data[2], -self.data[3]
            ]
        }
    }
}

impl Sub<Mat2> for Mat2 {
    type Output = Mat2;
    fn sub(self, rhs: Mat2) -> Self::Output {
        self + -rhs
    }
}

impl Mat2 {
    pub const fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self {
            data: [a, b, c, d]
        }
    }

    pub fn rotation(angle: f32) -> Self {
        let sin = angle.sin();
        let cos = angle.cos();
        Self {
            data: [
                cos, -sin,
                sin, cos
            ]
        }
    }

    pub fn a(&self) -> f32 {
        self.data[0]
    }

    pub fn b(&self) -> f32 {
        self.data[1]
    }

    pub fn c(&self) -> f32 {
        self.data[2]
    }

    pub fn d(&self) -> f32 {
        self.data[3]
    }

    pub fn det(&self) -> f32 {
        self.data[0] * self.data[3] - self.data[1] * self.data[2]
    }

    pub fn inv(&self) -> Self {
        let det = self.det();
        Mat2::new(self.data[3], -self.data[1], -self.data[2], self.data[0]) / det
    }

    pub fn i(&self) -> Vec2 {
        Vec2::new(self.data[0], self.data[2])
    }

    pub fn j(&self) -> Vec2 {
        Vec2::new(self.data[1], self.data[3])
    }
}