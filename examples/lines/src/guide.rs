use crate::Line;
use carbide::draw::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Guide {
    Point(Position),
    Vertical(f64),
    Horizontal(f64),
    Directional(Line),
}
