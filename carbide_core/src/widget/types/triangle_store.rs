use crate::Color;
use crate::color::Rgba;
use crate::draw::{Dimension, Position, Rect};
use crate::draw::shape::triangle::Triangle;
use crate::prelude::Primitive;
use crate::render::PrimitiveKind;

#[derive(PartialEq, Clone, Debug)]
pub struct TriangleStore {
    pub latest_stroke_position: Position,
    pub latest_stroke_dimensions: Dimension,

    pub latest_fill_position: Position,
    pub latest_fill_dimensions: Dimension,

    pub stroke_triangles: Vec<Triangle<Position>>,
    pub fill_triangles: Vec<Triangle<Position>>,
}

impl TriangleStore {
    pub fn new() -> TriangleStore {
        TriangleStore {
            latest_stroke_position: Position::new(0.0, 0.0),
            latest_stroke_dimensions: Dimension::new(0.0, 0.0),
            latest_fill_position: Position::new(0.0, 0.0),
            latest_fill_dimensions: Dimension::new(0.0, 0.0),
            stroke_triangles: vec![],
            fill_triangles: vec![],
        }
    }

    pub fn diff_stroke(&self, position: Position, dimensions: Dimension) -> bool {
        self.latest_stroke_position != position || self.latest_stroke_dimensions != dimensions
    }

    // Todo: Maybe translate on position change instead of re-tessellating
    pub fn diff_fill(&self, position: Position, dimensions: Dimension) -> bool {
        self.latest_fill_position != position || self.latest_fill_dimensions != dimensions
    }

    pub fn stroke_triangles(&self) -> Vec<Triangle<Position>> {
        self.stroke_triangles.clone()
    }

    pub fn fill_triangles(&self) -> Vec<Triangle<Position>> {
        self.fill_triangles.clone()
    }

    pub fn stroke_triangles_mut(&mut self) -> &mut Vec<Triangle<Position>> {
        &mut self.stroke_triangles
    }

    pub fn fill_triangles_mut(&mut self) -> &mut Vec<Triangle<Position>> {
        &mut self.fill_triangles
    }

    pub fn set_stroke_triangles(&mut self, triangles: &Vec<Triangle<Position>>) {
        self.stroke_triangles = triangles.clone()
    }

    pub fn set_fill_triangles(&mut self, triangles: &Vec<Triangle<Position>>) {
        self.fill_triangles = triangles.clone()
    }

    pub fn insert_primitives(&self, primitives: &mut Vec<Primitive>, fill_color: Color, stroke_color: Color) {
        if self.fill_triangles.len() > 0 {
            primitives.push(Primitive {
                kind: PrimitiveKind::Gradient(
                    self.fill_triangles.clone(),
                ),
                rect: Rect::new(self.latest_fill_position, self.latest_fill_dimensions),
            });
            /*primitives.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor {
                    color: Rgba::from(fill_color),
                    triangles: self.fill_triangles.clone(),
                },
                rect: Rect::new(self.latest_fill_position, self.latest_fill_dimensions),
            });*/
        }

        if self.stroke_triangles.len() > 0 {
            primitives.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor {
                    color: Rgba::from(stroke_color),
                    triangles: self.stroke_triangles.clone(),
                },
                rect: Rect::new(self.latest_stroke_position, self.latest_stroke_dimensions),
            });
        }
    }

    pub fn get_primitives(&self, fill_color: Color, stroke_color: Color) -> Vec<Primitive> {
        let mut res = vec![];
        if self.fill_triangles.len() > 0 {
            res.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor {
                    color: Rgba::from(fill_color),
                    triangles: self.fill_triangles.clone(),
                },
                rect: Rect::new(self.latest_fill_position, self.latest_fill_dimensions),
            });
        }

        if self.stroke_triangles.len() > 0 {
            res.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor {
                    color: Rgba::from(stroke_color),
                    triangles: self.stroke_triangles.clone(),
                },
                rect: Rect::new(self.latest_stroke_position, self.latest_stroke_dimensions),
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
