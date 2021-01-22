use crate::Point;
use crate::draw::shape::triangle::Triangle;
use crate::position::Dimensions;

#[derive(PartialEq, Clone, Debug)]
pub struct TriangleStore {
    pub position: Point,
    pub dimensions: Dimensions,
    pub triangles: Vec<Triangle<Point>>
}

impl TriangleStore {
    pub fn new() -> TriangleStore {
        TriangleStore {
            position: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            triangles: vec![]
        }
    }

    pub fn diff(&self, position: Point, dimensions: Dimensions) -> bool {
        self.position != position || self.dimensions != dimensions
    }

    pub fn triangles(&self) -> Vec<Triangle<Point>> {
        self.triangles.clone()
    }

    pub fn set_triangles(&mut self, triangles: &Vec<Triangle<Point>>) {
        self.triangles = triangles.clone()
    }
}