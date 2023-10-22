use std::fmt::{Debug, Formatter};

use lyon::algorithms::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};
use carbide_core::CommonWidgetImpl;
use carbide_core::render::{RenderContext};

use carbide_macro::{carbide_default_builder2};

use crate::draw::{Dimension, Position, Rect, Color, Scalar};
use crate::draw::draw_style::DrawStyle;
use crate::draw::shape::triangle::Triangle;
use crate::environment::{Environment, EnvironmentColor};
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::{NewStateSync, ReadState, StateContract, TState, ValueState};
use crate::widget::{CommonWidget, PrimitiveStore, Shape, ShapeStyle, StrokeStyle, AnyWidget, WidgetExt, WidgetId, Widget};
use crate::widget::canvas::{Context, ShapeStyleWithOptions};

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Canvas<C>
where
    C: CanvasContext
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    context: C,
}

pub trait CanvasContext: Clone + 'static {
    fn call(&self, area: Rect, context: Context, env: &mut Environment) -> Context;
}

impl<T> CanvasContext for T where T: Fn(Rect, Context, &mut Environment) -> Context + Clone + 'static {
    fn call(&self, area: Rect, context: Context, env: &mut Environment) -> Context {
        self(area, context, env)
    }
}

type DefaultCanvasContext = fn(Rect, Context, &mut Environment) -> Context;

impl Canvas<DefaultCanvasContext> {

    #[carbide_default_builder2]
    pub fn new<C: CanvasContext>(context: C) -> Canvas<C> {
        Canvas {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),

            context,
        }
    }
}

impl<C: CanvasContext> Canvas<C> {

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
            kind: PrimitiveKind::Geometry {
                color,
                triangles: Triangle::from_point_list(points),
            },
            bounding_box: Rect::new(self.position, self.dimension),
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
            kind: PrimitiveKind::Geometry {
                color,
                triangles: Triangle::from_point_list(points),
            },
            bounding_box: Rect::new(self.position, self.dimension),
        }
    }

    pub fn get_fill_geometry(&self, path: Path, fill_options: FillOptions) -> Vec<Triangle<Position>> {
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

        Triangle::from_point_list(points)
    }

    pub fn get_stroke_geometry(
        &self,
        path: Path,
        stroke_options: StrokeOptions,
    ) -> Vec<Triangle<Position>> {
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

        Triangle::from_point_list(points)
    }
}

impl<C: CanvasContext> CommonWidget for Canvas<C> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl<C: CanvasContext> Shape for Canvas<C> {
    fn get_triangle_store_mut(&mut self) -> &mut PrimitiveStore {
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

        let context = (self.context).call(rectangle, context, env);

        let paths = context.to_paths(self.position(), env);
        let mut prims = vec![];

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, mut color) => {
                    color.sync(env);
                    prims.push(self.get_fill_prim(path, fill_options, Color::Rgba(1.0, 0.0, 0.0, 1.0)));
                    //color.release_state(env);
                }
                ShapeStyleWithOptions::Stroke(stroke_options, mut color) => {
                    color.sync(env);
                    prims.push(self.get_stroke_prim(path, stroke_options, Color::Rgba(1.0, 0.0, 0.0, 1.0)));
                    //color.release_state(env);
                }
            }
        }

        let mut res_triangle_list = vec![];

        for prim in prims {
            match prim.kind {
                PrimitiveKind::Geometry { triangles, .. } => {
                    res_triangle_list.extend(triangles);
                }
                _ => (),
            }
        }

        res_triangle_list
    }
}

impl<C: CanvasContext> Render for Canvas<C> {
    fn render(&mut self, render_context: &mut RenderContext, env: &mut Environment) {
        let context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());
        let context = self.context.call(rectangle, context, env);

        let paths = context.to_paths(self.position(), env);

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, style) => {
                    render_context.style(style.convert(self.position, self.dimension), |this| {
                        this.geometry(&self.get_fill_geometry(path, fill_options))
                    })
                }
                ShapeStyleWithOptions::Stroke(stroke_options, style) => {
                    render_context.style(style.convert(self.position, self.dimension), |this| {
                        this.geometry(&self.get_stroke_geometry(path, stroke_options))
                    })
                }
            }
        }
    }
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        let context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());
        let context = self.context.call(rectangle, context, env);

        let paths = context.to_paths(self.position(), env);

        todo!()

        /*for (path, options) in paths {
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
        }*/
    }
}

impl<C: CanvasContext> Debug for Canvas<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Canvas").finish()
    }
}

impl<C: CanvasContext> WidgetExt for Canvas<C> {}
