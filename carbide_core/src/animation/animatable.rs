use crate::draw::{Dimension, Position};
use crate::Color;

/// This trait is the base for things that are animatable. To animate a value in an [Animation]
/// you can either provide a value that is animatable or provide a value with a custom interpolation.
/// Animatable is only implemented for some common types by default. These include f32, f64 and
/// [Color]. By default the [Color] is interpolated as a blend in rgb space.
/// When implementing animatable you should think of the animation as linear. Easing is implemented
/// separately. If you start doing easing during interpolation you will have issues with easings
/// possibly being applied twice.
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

impl Animatable<Position> for Position {
    fn interpolate(&self, other: &Position, percentage: f64) -> Position {
        Position {
            x: self.x.interpolate(&other.x, percentage),
            y: self.y.interpolate(&other.y, percentage),
        }
    }
}

impl Animatable<Dimension> for Dimension {
    fn interpolate(&self, other: &Dimension, percentage: f64) -> Dimension {
        Dimension {
            width: self.width.interpolate(&other.width, percentage),
            height: self.height.interpolate(&other.height, percentage),
        }
    }
}
