use carbide_core::draw::{Angle, Position};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StrokeVertex {
    pub position: Position,

    pub start: Position,
    pub end: Position,

    pub start_angle: Angle,
    pub end_angle: Angle,

    pub width: f32,
    pub offset: f32,
}