use lyon::algorithms::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

use crate::color::Rgba;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::draw::shape::triangle::Triangle;
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::widget::canvas::{Context, ShapeStyleWithOptions};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Canvas {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    #[state]
    color: ColorState,
    //prim_store: Vec<Primitive>,
    context: fn(Rect, Context) -> Context,
}

impl Canvas {
    pub fn new(context: fn(Rect, Context) -> Context) -> Box<Self> {
        Box::new(Canvas {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            color: EnvironmentColor::Accent.into(),
            //prim_store: vec![],
            context,
        })
    }

    pub fn get_stroke_prim(
        &self,
        path: Path,
        stroke_options: StrokeOptions,
        color: Color,
    ) -> Primitive {
        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &stroke_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                        let point = vertex.position().to_array();
                        Position::new(point[0] as Scalar, point[1] as Scalar)
                    }),
                )
                .unwrap();
        }

        let point_iter = geometry
            .indices
            .iter()
            .map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Position> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor {
                color: Rgba::from(color),
                triangles: Triangle::from_point_list(points),
            },
            rect: Rect::new(self.position, self.dimension),
        }
    }

    pub fn get_fill_prim(&self, path: Path, fill_options: FillOptions, color: Color) -> Primitive {
        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &fill_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        let point = vertex.position().to_array();
                        Position::new(point[0] as Scalar, point[1] as Scalar)
                    }),
                )
                .unwrap();
        }

        let point_iter = geometry
            .indices
            .iter()
            .map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Position> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor {
                color: Rgba::from(color),
                triangles: Triangle::from_point_list(points),
            },
            rect: Rect::new(self.position, self.dimension),
        }
    }
}

CommonWidgetImpl!(Canvas, self, id: self.id, position: self.position, dimension: self.dimension);

impl Shape for Canvas {
    fn get_triangle_store_mut(&mut self) -> &mut TriangleStore {
        todo!()
    }

    fn get_stroke_style(&self) -> StrokeStyle {
        todo!()
    }

    fn get_shape_style(&self) -> ShapeStyle {
        todo!()
    }

    fn triangles(&mut self, env: &mut Environment) -> Vec<Triangle<Position>> {
        let context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());
        let context = (self.context)(rectangle, context);

        let paths = context.to_paths(self.position());
        let mut prims = vec![];

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, mut color) => {
                    color.sync(env);
                    prims.push(self.get_fill_prim(path, fill_options, *color.value()));
                    //color.release_state(env);
                }
                ShapeStyleWithOptions::Stroke(stroke_options, mut color) => {
                    color.sync(env);
                    prims.push(self.get_stroke_prim(path, stroke_options, *color.value()));
                    //color.release_state(env);
                }
            }
        }

        let mut res_triangle_list = vec![];

        for prim in prims {
            match prim.kind {
                PrimitiveKind::TrianglesSingleColor { triangles, .. } => {
                    res_triangle_list.extend(triangles);
                }
                _ => (),
            }
        }

        res_triangle_list
    }
}

impl Render for Canvas {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        let context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());
        let context = (self.context)(rectangle, context);

        let paths = context.to_paths(self.position());

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, mut color) => {
                    color.sync(env);
                    primitives.push(self.get_fill_prim(path, fill_options, *color.value()));
                    //color.release_state(env);
                }
                ShapeStyleWithOptions::Stroke(stroke_options, mut color) => {
                    color.sync(env);
                    primitives.push(self.get_stroke_prim(path, stroke_options, *color.value()));
                    //color.release_state(env);
                }
            }
        }
    }
}

impl WidgetExt for Canvas {}
