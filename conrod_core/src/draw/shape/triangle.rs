use draw::shape::vertex::Vertex;
use ::{Point, color};
use widget::primitive::shape::triangles::ColoredPoint;

/// A single triangle described by three vertices.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Triangle<V>(pub [V; 3])
    where V: Vertex;

impl<V> Triangle<V>
    where V: Vertex,
{
    /// Shift the triangle by the given amount by adding it onto the position of each point.
    pub fn add(self, amount: Point) -> Self {
        let a = self[0].add(amount);
        let b = self[1].add(amount);
        let c = self[2].add(amount);
        Triangle([a, b, c])
    }

    /// The three points that make up the triangle.
    pub fn points(self) -> [Point; 3] {
        [self[0].point(), self[1].point(), self[2].point()]
    }
}

impl Triangle<Point> {
    /// Convert the `Triangle<Point>` to a `Triangle<ColoredPoint>`.
    pub fn color(self, a: color::Rgba, b: color::Rgba, c: color::Rgba) -> Triangle<ColoredPoint> {
        Triangle([(self[0], a), (self[1], b), (self[2], c)])
    }

    /// Convert the `Triangle<Point>` to a `Triangle<ColoredPoint>` using the given color.
    pub fn color_all(self, color: color::Rgba) -> Triangle<ColoredPoint> {
        Triangle([(self[0], color), (self[1], color), (self[2], color)])
    }
}

impl<V> std::ops::Deref for Triangle<V>
    where V: Vertex,
{
    type Target = [V; 3];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> From<[V; 3]> for Triangle<V>
    where V: Vertex,
{
    fn from(points: [V; 3]) -> Self {
        Triangle(points)
    }
}

impl<V> From<(V, V, V)> for Triangle<V>
    where V: Vertex,
{
    fn from((a, b, c): (V, V, V)) -> Self {
        Triangle([a, b, c])
    }
}

impl<V> Into<[V; 3]> for Triangle<V>
    where V: Vertex,
{
    fn into(self) -> [V; 3] {
        self.0
    }
}

impl<V> Into<(V, V, V)> for Triangle<V>
    where V: Vertex,
{
    fn into(self) -> (V, V, V) {
        (self[0], self[1], self[2])
    }
}
