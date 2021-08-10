use crate::draw::Position;
use crate::widget::ColoredPoint;

/// Types used as vertices that make up a list of triangles.
pub trait Vertex: Clone + Copy + PartialEq {
    /// The x y location of the vertex.
    fn point(&self) -> Position;
    /// Add the given vector onto the position of self and return the result.
    fn add_vertex(self, other: Position) -> Self;
}

impl Vertex for Position {
    fn point(&self) -> Position {
        *self
    }

    fn add_vertex(self, add: Position) -> Self {
        self + add
    }
}

impl Vertex for ColoredPoint {
    fn point(&self) -> Position {
        self.0
    }

    fn add_vertex(self, add: Position) -> Self {
        let (p, c) = self;
        (p + add, c)
    }
}
