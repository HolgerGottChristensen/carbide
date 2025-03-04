use carbide::draw::Position;
use crate::draw::path::Path;
use crate::draw::{DrawOptions, Rect, Scalar};
use crate::widget::CornerRadii;

#[derive(Clone, Debug, PartialEq)]
pub enum DrawShape {
    Rectangle(Rect),
    Capsule(Rect),
    RoundedRectangle(Rect, CornerRadii),
    Circle(Position, Scalar),
    Ellipse(Rect),
    Path(Path),
    Single(Box<(DrawShape, DrawOptions)>),
    Multiple(Vec<(DrawShape, DrawOptions)>),
}