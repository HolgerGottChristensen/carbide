use crate::draw::Position;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StrokeVertex {
    pub position: Position,

    pub start: Position,
    pub middle: Position,
    pub end: Position,

    pub width: f32,
    pub offset: f32,
}