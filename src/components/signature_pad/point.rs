use super::utils::PointExt;
use dioxus::prelude::*;
use std::ops::{Add, Mul, Sub};
use web_sys::HtmlCanvasElement;
#[derive(Clone, Debug, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
}
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            pressure: 0.5,
        }
    }
    pub fn new_with_pressure(x: f64, y: f64, pressure: f32) -> Self {
        Self { x, y, pressure }
    }
    pub fn from_event(event: &PointerEvent, canvas: &HtmlCanvasElement) -> Self {
        let coords = event.data().client_coordinates();
        let rect = canvas.get_bounding_client_rect();
        let scale_x = canvas.width() as f64 / rect.width();
        let scale_y = canvas.height() as f64 / rect.height();
        let x = (coords.x - rect.left()) * scale_x;
        let y = (coords.y - rect.top()) * scale_y;
        Self::new_with_pressure(x, y, event.data().pressure())
    }
}
impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            pressure: self.pressure,
        }
    }
}
impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            pressure: self.pressure,
        }
    }
}
impl Mul<f64> for Point {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            pressure: self.pressure,
        }
    }
}
impl PointExt for Point {
    fn addp(self, other: Self) -> Self {
        self + other
    }
    fn subp(self, other: Self) -> Self {
        self - other
    }
    fn mulp(self, other: f64) -> Self {
        self * other
    }
    fn per(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
            pressure: self.pressure,
        }
    }
    fn dpr(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    fn equal_to(self, other: Self) -> bool {
        (self.x - other.x).abs() < f64::EPSILON && (self.y - other.y).abs() < f64::EPSILON
    }
    fn as_vector(self) -> [f64; 2] {
        [self.x, self.y]
    }
}
