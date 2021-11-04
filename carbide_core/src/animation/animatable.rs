use crate::Color;

pub trait Animatable<T> {
    fn interpolate(&self, other: &T, percentage: f64) -> T;
}


impl Animatable<f32> for f32 {
    fn interpolate(&self, other: &f32, percentage: f64) -> f32 {
        *self * (1.0 - percentage as f32) + *other * percentage as f32
    }
}

impl Animatable<f64> for f64 {
    fn interpolate(&self, other: &f64, percentage: f64) -> f64 {
        *self * (1.0 - percentage) + *other * percentage
    }
}

impl Animatable<Color> for Color {
    fn interpolate(&self, other: &Color, percentage: f64) -> Color {
        Color::rgba_blend(self, other, percentage)
    }
}
