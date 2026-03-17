use crate::draw::Position;
use crate::draw::path::Path;
use crate::draw::{DrawOptions, Rect, Scalar};
use crate::widget::CornerRadii;

#[derive(Clone, Debug, PartialEq)]
pub enum CompositeDrawShape {
    Zero,
    One(DrawShape, DrawOptions),
    Many(Vec<(DrawShape, DrawOptions)>)
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrawShape {
    Rectangle(Rect), // TODO: Could be rectangle with radii 0.0
    Capsule(Rect),
    RoundedRectangle(Rect, CornerRadii),
    Circle(Position, Scalar), // TODO: Could be Ellipse with both radii the same
    Ellipse(Rect),
    Line(Position, Position),
    Path(Path),
}