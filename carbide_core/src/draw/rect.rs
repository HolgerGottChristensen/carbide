use crate::draw::dimension::Dimension;
use crate::draw::Position;
use crate::Point;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct Rect {
    pub position: Position,
    pub dimension: Dimension,
}