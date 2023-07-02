use std::ops::*;

use crate::drawing::val_from_rgb;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2<T: Num> {
    pub x: T,
    pub y: T,
}
pub trait Num {}
impl Num for i32 {}
impl Num for f32 {}
impl Num for u32 {}

impl<T: Num> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Vec2 { x, y }
    }
    pub fn from_tuple(tup: (T, T)) -> Self {
        Vec2 { x: tup.0, y: tup.1 }
    }
}

impl Vec2<i32> {
    pub fn as_f32(self) -> Vec2<f32> {
        Vec2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
    pub fn length(&self) -> f32 {
        ((self.x * self.x + self.y * self.y) as f32).sqrt()
    }
}

impl Vec2<f32> {
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn rotate(self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2::new(cos, sin) * self.x + Vec2::new(-sin, cos) * self.y
    }
    pub fn normalize(self) -> Self {
        let len = self.length();
        self / len
    }
    pub fn as_i32(self) -> Vec2<i32> {
        Vec2 {
            x: self.x as i32,
            y: self.y as i32,
        }
    }
}

impl<T: Num + Add<Output = T>> Add for Vec2<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl<T: Num + Sub<Output = T>> Sub for Vec2<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl<T: Num + Div<Output = T> + Copy> Div<T> for Vec2<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Num + Mul<Output = T> + Copy> Mul<T> for Vec2<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

pub fn set_value_brightness(val: u32, brightness: f32) -> u32 {
    let b = val & 0xFF;
    let g = val >> 8 & 0xFF;
    let r = val >> 16 & 0xFF;
    val_from_rgb(
        (r as f32 * brightness) as u32,
        (g as f32 * brightness) as u32,
        (b as f32 * brightness) as u32,
    )
}
