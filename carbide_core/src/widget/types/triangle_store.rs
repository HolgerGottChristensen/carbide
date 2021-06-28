use crate::{Color, OldRect, Point};
use crate::color::Rgba;
use crate::draw::shape::triangle::Triangle;
use crate::position::Dimensions;
use crate::prelude::Primitive;
use crate::render::primitive_kind::PrimitiveKind;

#[derive(PartialEq, Clone, Debug)]
pub struct TriangleStore {
    pub latest_stroke_position: Point,
    pub latest_stroke_dimensions: Dimensions,

    pub latest_fill_position: Point,
    pub latest_fill_dimensions: Dimensions,

    pub stroke_triangles: Vec<Triangle<Point>>,
    pub fill_triangles: Vec<Triangle<Point>>,
}

impl TriangleStore {
    pub fn new() -> TriangleStore {
        TriangleStore {
            latest_stroke_position: [0.0, 0.0],
            latest_stroke_dimensions: [0.0, 0.0],
            latest_fill_position: [0.0, 0.0],
            latest_fill_dimensions: [0.0, 0.0],
            stroke_triangles: vec![],
            fill_triangles: vec![],
        }
    }

    pub fn diff_stroke(&self, position: Point, dimensions: Dimensions) -> bool {
        self.latest_stroke_position != position || self.latest_stroke_dimensions != dimensions
    }

    // Todo: Maybe translate on position change instead of retessellating
    pub fn diff_fill(&self, position: Point, dimensions: Dimensions) -> bool {
        self.latest_fill_position != position || self.latest_fill_dimensions != dimensions
    }

    pub fn stroke_triangles(&self) -> Vec<Triangle<Point>> {
        self.stroke_triangles.clone()
    }

    pub fn fill_triangles(&self) -> Vec<Triangle<Point>> {
        self.fill_triangles.clone()
    }

    pub fn set_stroke_triangles(&mut self, triangles: &Vec<Triangle<Point>>) {
        self.stroke_triangles = triangles.clone()
    }

    pub fn set_fill_triangles(&mut self, triangles: &Vec<Triangle<Point>>) {
        self.fill_triangles = triangles.clone()
    }

    pub fn get_primitives(&self, fill_color: Color, stroke_color: Color) -> Vec<Primitive> {
        let mut res = vec![];
        if self.fill_triangles.len() > 0 {
            res.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(fill_color), triangles: self.fill_triangles.clone() },
                rect: OldRect::new(self.latest_fill_position, self.latest_fill_dimensions),
            });
        }

        if self.stroke_triangles.len() > 0 {
            res.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(stroke_color), triangles: self.stroke_triangles.clone() },
                rect: OldRect::new(self.latest_stroke_position, self.latest_stroke_dimensions),
            });
        }

        res
    }
}

impl Default for TriangleStore {
    fn default() -> Self {
        TriangleStore::new()
    }
}