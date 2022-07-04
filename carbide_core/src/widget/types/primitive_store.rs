use crate::color::Rgba;
use crate::draw::draw_gradient::DrawGradient;
use crate::draw::shape::triangle::Triangle;
use crate::draw::{Dimension, Position, Rect};
use crate::prelude::Primitive;
use crate::render::PrimitiveKind;
use crate::widget::types::advanced_color::AdvancedColor;
use crate::Color;

/// A storage container for primitives that can be used to cache tessellated shapes.
#[derive(PartialEq, Clone, Debug)]
pub struct PrimitiveStore {
    pub latest_stroke_position: Position,
    pub latest_stroke_dimensions: Dimension,
    pub latest_stoke_color: Option<AdvancedColor>,
    //pub stroke_primitive: Option<Primitive>,
    pub latest_fill_position: Position,
    pub latest_fill_dimensions: Dimension,
    pub latest_fill_color: Option<AdvancedColor>,
    //pub fill_primitive: Option<Primitive>,
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

    pub fn insert_primitives(
        &self,
        primitives: &mut Vec<Primitive>,
        fill_color: AdvancedColor,
        stroke_color: AdvancedColor,
        position: Position,
        dimension: Dimension,
    ) {
        if self.fill_triangles.len() > 0 {
            let fill_color = fill_color.into();

            match fill_color {
                AdvancedColor::Color(c) => {
                    primitives.push(Primitive {
                        kind: PrimitiveKind::TrianglesSingleColor {
                            color: c,
                            triangles: self.fill_triangles.clone(),
                        },
                        bounding_box: Rect::new(
                            self.latest_fill_position,
                            self.latest_fill_dimensions,
                        ),
                    });
                }
                AdvancedColor::SingleGradient(g) => {
                    primitives.push(Primitive {
                        kind: PrimitiveKind::Gradient(
                            self.fill_triangles.clone(),
                            DrawGradient::convert(g, position, dimension),
                        ),
                        bounding_box: Rect::new(
                            self.latest_fill_position,
                            self.latest_fill_dimensions,
                        ),
                    });
                }
                AdvancedColor::MultiGradient(_) => {}
            }
        }

        if self.stroke_triangles.len() > 0 {
            let stroke_color = stroke_color.into();

            match stroke_color {
                AdvancedColor::Color(c) => {
                    primitives.push(Primitive {
                        kind: PrimitiveKind::TrianglesSingleColor {
                            color: c,
                            triangles: self.stroke_triangles.clone(),
                        },
                        bounding_box: Rect::new(
                            self.latest_stroke_position,
                            self.latest_stroke_dimensions,
                        ),
                    });
                }
                AdvancedColor::SingleGradient(g) => {
                    primitives.push(Primitive {
                        kind: PrimitiveKind::Gradient(
                            self.stroke_triangles.clone(),
                            DrawGradient::convert(g, position, dimension),
                        ),
                        bounding_box: Rect::new(
                            self.latest_stroke_position,
                            self.latest_stroke_dimensions,
                        ),
                    });
                }
                AdvancedColor::MultiGradient(_) => {}
            }
        }
    }

    pub fn get_primitives(&self, fill_color: Color, stroke_color: Color) -> Vec<Primitive> {
        let mut res = vec![];
        if self.fill_triangles.len() > 0 {
            res.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor {
                    color: fill_color,
                    triangles: self.fill_triangles.clone(),
                },
                bounding_box: Rect::new(self.latest_fill_position, self.latest_fill_dimensions),
            });
        }

        if self.stroke_triangles.len() > 0 {
            res.push(Primitive {
                kind: PrimitiveKind::TrianglesSingleColor {
                    color: stroke_color,
                    triangles: self.stroke_triangles.clone(),
                },
                bounding_box: Rect::new(self.latest_stroke_position, self.latest_stroke_dimensions),
            });
        }

        res
    }
}

impl Default for PrimitiveStore {
    fn default() -> Self {
        PrimitiveStore::new()
    }
}
