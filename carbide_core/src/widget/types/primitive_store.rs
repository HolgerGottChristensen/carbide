use crate::draw::{Dimension, Position};
use crate::draw::shape::triangle::Triangle;
use crate::widget::types::advanced_color::AdvancedColor;

/// A storage container for primitives that can be used to cache tessellated shapes.
#[derive(PartialEq, Clone, Debug)]
pub struct PrimitiveStore {
    pub latest_stroke_position: Position,
    pub latest_stroke_dimensions: Dimension,
    pub latest_stoke_color: Option<AdvancedColor>,
    pub latest_fill_position: Position,
    pub latest_fill_dimensions: Dimension,
    pub latest_fill_color: Option<AdvancedColor>,
    pub stroke_triangles: Vec<Triangle<Position>>,
    pub fill_triangles: Vec<Triangle<Position>>,
}

impl PrimitiveStore {
    pub fn new() -> PrimitiveStore {
        PrimitiveStore {
            latest_stroke_position: Position::new(0.0, 0.0),
            latest_stroke_dimensions: Dimension::new(0.0, 0.0),
            latest_stoke_color: None,
            //stroke_primitive: None,
            latest_fill_position: Position::new(0.0, 0.0),
            latest_fill_dimensions: Dimension::new(0.0, 0.0),
            latest_fill_color: None,
            //fill_primitive: None,
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
}

impl Default for PrimitiveStore {
    fn default() -> Self {
        PrimitiveStore::new()
    }
}
