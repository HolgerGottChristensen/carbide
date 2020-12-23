use crate::Point;
use crate::utils::vec2_add;
use crate::widget::primitive::shape::triangles::ColoredPoint;

/// Types used as vertices that make up a list of triangles.
pub trait Vertex: Clone + Copy + PartialEq {
    /// The x y location of the vertex.
    fn point(&self) -> Point;
    /// Add the given vector onto the position of self and return the result.
    fn add(self, other: Point) -> Self;
}

impl Vertex for Point {
    fn point(&self) -> Point {
        *self
    }
    fn add(self, add: Point) -> Self {
        vec2_add(self, add)
    }
}

impl Vertex for ColoredPoint {
    fn point(&self) -> Point {
        self.0
    }
    fn add(self, add: Point) -> Self {
        let (p, c) = self;
        (vec2_add(p, add), c)
    }
}