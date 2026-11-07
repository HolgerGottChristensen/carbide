use carbide_core::draw::Position;
use carbide_core::widget::ColoredPoint;
use crate::stroke_vertex::StrokeVertex;

/// Types used as vertices that make up a list of triangles.
pub trait Vertex: Clone + Copy + PartialEq {
    /// The x y location of the vertex.
    fn point(&self) -> Position;
    /// Add the given vector onto the position of self and return the result.
    fn add_vertex(self, other: Position) -> Self;
    fn offset(&mut self, other: Position);
}

impl Vertex for Position {
    fn point(&self) -> Position {
        *self
    }

    fn add_vertex(self, add: Position) -> Self {
        self + add
    }

    fn offset(&mut self, other: Position) {
        *self = *self + other;
    }
}

impl Vertex for (Position, Position) {
    fn point(&self) -> Position {
        todo!()
    }

    fn add_vertex(self, _add: Position) -> Self {
        todo!()
    }

    fn offset(&mut self, _other: Position) {
        todo!()
    }
}

impl Vertex for StrokeVertex {
    fn point(&self) -> Position {
        todo!()
    }

    fn add_vertex(self, _add: Position) -> Self {
        todo!()
    }

    fn offset(&mut self, _other: Position) {
        todo!()
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

    fn offset(&mut self, other: Position) {
        let (p, _) = self;
        *p = *p + other;
    }
}
