use crate::color;
use crate::draw::Position;
use crate::draw::shape::vertex::Vertex;
use crate::widget::ColoredPoint;

/// A single triangle described by three vertices.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Triangle<V>(pub [V; 3])
    where
        V: Vertex;

impl<V> Triangle<V>
    where
        V: Vertex,
{
    /// Shift the triangle by the given amount by adding it onto the position of each point.
    pub fn add(self, amount: Position) -> Self {
        let a = self[0].add_vertex(amount);
        let b = self[1].add_vertex(amount);
        let c = self[2].add_vertex(amount);
        Triangle([a, b, c])
    }

    pub fn offset(&mut self, amount: Position) {
        (self.0[0]).offset(amount);
        (self.0[1]).offset(amount);
        (self.0[2]).offset(amount);
    }

    /// The three points that make up the triangle.
    pub fn points(self) -> [Position; 3] {
        [self[0].point(), self[1].point(), self[2].point()]
    }
}

impl Triangle<Position> {
    /// Convert the `Triangle<Point>` to a `Triangle<ColoredPoint>`.
    pub fn color(self, a: color::Rgba, b: color::Rgba, c: color::Rgba) -> Triangle<ColoredPoint> {
        Triangle([(self[0], a), (self[1], b), (self[2], c)])
    }

    /// Convert the `Triangle<Point>` to a `Triangle<ColoredPoint>` using the given color.
    pub fn color_all(self, color: color::Rgba) -> Triangle<ColoredPoint> {
        Triangle([(self[0], color), (self[1], color), (self[2], color)])
    }

    pub fn from_point_list(points: Vec<Position>) -> Vec<Triangle<Position>> {
        let len = points.len();

        if len == 0 {
            return vec![];
        }

        if len % 3 != 0 {
            panic!("The vector of points can not be converted to a list of triangles because its length is not divisible by three")
        }

        let mut res = vec![];

        for i in (0..len).step_by(3) {
            res.push((points[i], points[i + 1], points[i + 2]).into())
        }

        res
    }
}

impl<V> std::ops::Deref for Triangle<V>
    where
        V: Vertex,
{
    type Target = [V; 3];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> From<[V; 3]> for Triangle<V>
    where
        V: Vertex,
{
    fn from(points: [V; 3]) -> Self {
        Triangle(points)
    }
}

impl<V> From<(V, V, V)> for Triangle<V>
    where
        V: Vertex,
{
    fn from((a, b, c): (V, V, V)) -> Self {
        Triangle([a, b, c])
    }
}

impl<V> Into<[V; 3]> for Triangle<V>
    where
        V: Vertex,
{
    fn into(self) -> [V; 3] {
        self.0
    }
}

impl<V> Into<(V, V, V)> for Triangle<V>
    where
        V: Vertex,
{
    fn into(self) -> (V, V, V) {
        (self[0], self[1], self[2])
    }
}
