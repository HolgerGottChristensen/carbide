use lyon::lyon_tessellation::path::path::Builder;
use lyon::lyon_tessellation::StrokeAlignment;
use lyon::math::Box2D;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, Side, StrokeOptions,
    StrokeTessellator, StrokeVertex as LyonStrokeVertex, VertexBuffers,
};
use lyon::tessellation::path::Path;

pub use capsule::*;
use carbide::draw::Dimension;
pub use circle::*;
pub use ellipse::*;
pub use rectangle::*;
pub use rounded_rectangle::*;

use crate::color::Color;
use crate::draw::{DrawStyle, ImageId, NOOPImageContext, Position, Rect, Scalar, StrokeDashPattern};
use crate::draw::shape::stroke_vertex::StrokeVertex;
use crate::draw::shape::triangle::Triangle;
use crate::render::triangle_render_context::TriangleRenderContext;
use crate::environment::{Environment, EnvironmentStack};
use crate::render::{CarbideTransform, InnerRenderContext, Layer, LayerId, NoopLayer, RenderContext};
use crate::text::{InnerTextContext, NOOPTextContext, TextId};
use crate::widget::{AnyWidget, FilterId};
use crate::widget::types::{PrimitiveStore, ShapeStyle, StrokeStyle};

mod capsule;
mod circle;
mod ellipse;
mod rectangle;
mod rounded_rectangle;

pub trait Shape: AnyWidget + 'static {
    fn get_triangle_store_mut(&mut self) -> &mut PrimitiveStore;
    fn get_stroke_style(&self) -> StrokeStyle;
    fn get_shape_style(&self) -> ShapeStyle;
    fn triangles(&mut self, env: &mut Environment) -> Vec<Triangle<Position>> {
        let mut geom = TriangleRenderContext(vec![]);
        self.render(&mut RenderContext {
                    render: &mut geom,
                    text: &mut NOOPTextContext,
                    image: &mut NOOPImageContext,
            env,
            env_stack: &mut EnvironmentStack::new(),
        });

        geom.0
    }
}

dyn_clone::clone_trait_object!(Shape);

impl AnyWidget for Box<dyn Shape> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

pub fn tessellate(shape: &mut dyn Shape, rectangle: &Box2D, path: &dyn Fn(&mut Builder, &Box2D)) {
    match shape.get_shape_style() {
        ShapeStyle::Default | ShapeStyle::Fill => {
            fill(path, shape, rectangle);
        }
        ShapeStyle::Stroke => {
            stroke(path, shape, rectangle);
        }
        ShapeStyle::FillAndStroke => {
            fill(path, shape, rectangle);
            stroke(path, shape, rectangle);
        }
    }
}

pub fn fill(path: &dyn Fn(&mut Builder, &Box2D), shape: &mut dyn Shape, rectangle: &Box2D) {
    let position = shape.position();
    let dimension = shape.dimension();
    let triangle_store = shape.get_triangle_store_mut();

    if dimension != triangle_store.latest_fill_dimensions {
        let mut builder = Path::builder();

        // Let the caller decide the geometry
        path(&mut builder, rectangle);

        let path = builder.build();

        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();

        let mut tessellator = FillTessellator::new();

        let fill_options = FillOptions::default();

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

        let triangles = Triangle::from_point_list(points);

        triangle_store.latest_fill_position = position;
        triangle_store.latest_fill_dimensions = dimension;
        triangle_store.set_fill_triangles(&triangles);
    } else if position != triangle_store.latest_fill_position {
        let offset = position - triangle_store.latest_fill_position;
        triangle_store
            .fill_triangles_mut()
            .iter_mut()
            .for_each(|t| {
                t.offset(offset);
            });
        triangle_store.latest_fill_position = position;
    }
}

pub fn stroke(path: &dyn Fn(&mut Builder, &Box2D), shape: &mut dyn Shape, rectangle: &Box2D) {
    let position = shape.position();
    let dimension = shape.dimension();
    let line_width = shape.get_stroke_style().get_line_width() as f32;
    let triangle_store = shape.get_triangle_store_mut();

    if dimension != triangle_store.latest_stroke_dimensions {
        let mut builder = Path::builder();

        // Let the caller decide the geometry
        path(&mut builder, &rectangle);

        let path = builder.build();

        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();

        let mut tessellator = StrokeTessellator::new();

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &StrokeOptions::default()
                        .with_line_width(line_width)
                        .with_alignment(StrokeAlignment::Positive),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: LyonStrokeVertex| {
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

        let triangles = Triangle::from_point_list(points);

        triangle_store.latest_stroke_position = position;
        triangle_store.latest_stroke_dimensions = dimension;
        triangle_store.set_stroke_triangles(&triangles);
    } else if position != triangle_store.latest_stroke_position {
        let offset = position - triangle_store.latest_stroke_position;
        triangle_store
            .stroke_triangles_mut()
            .iter_mut()
            .for_each(|t| {
                t.offset(offset);
            });
        triangle_store.latest_stroke_position = position;
    }
}
