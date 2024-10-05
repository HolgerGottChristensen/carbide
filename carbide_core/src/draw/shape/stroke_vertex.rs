use cgmath::Rad;
use lyon::math::{Angle, Point};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StrokeVertex {
    pub position: Point,

    pub start: Point,
    pub end: Point,

    pub start_angle: Angle,
    pub end_angle: Angle,

    pub width: f32,
    pub offset: f32,
}