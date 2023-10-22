//! A module encompassing the primitive 2D shape widgets.
use lyon::lyon_tessellation::path::path::Builder;
use lyon::math::Box2D;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, Side, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
};
use lyon::tessellation::path::Path;

pub use capsule::*;
pub use circle::*;
pub use ellipse::*;
pub use rectangle::*;
pub use rounded_rectangle::*;

use crate::draw::{Position, Scalar};
use crate::draw::shape::triangle::Triangle;
use crate::environment::Environment;
use crate::render::PrimitiveKind;
use crate::widget::types::PrimitiveStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;
use crate::widget::AnyWidget;

mod capsule;
mod circle;
mod ellipse;
mod rectangle;
mod rounded_rectangle;

pub trait Shape: AnyWidget + 'static {
    fn get_triangle_store_mut(&mut self) -> &mut PrimitiveStore;
    fn get_stroke_style(&self) -> StrokeStyle;
    fn get_shape_style(&self) -> ShapeStyle;
    // Todo: add primitives to before and after the shape.
    fn triangles(&mut self, env: &mut Environment) -> Vec<Triangle<Position>> {
        let mut primitives = vec![];
        self.get_primitives(&mut primitives, env);
        if primitives.len() >= 1 {
            match primitives.remove(0).kind {
                PrimitiveKind::Geometry { triangles, .. } => triangles,
                _ => {
                    panic!("Can only return triangles of PrimitiveKind::Geometry. This error might happen if you use a rectangle with content.")
                }
            }
        } else {
            vec![]
        }
    }
}

dyn_clone::clone_trait_object!(Shape);

impl AnyWidget for Box<dyn Shape> {}


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

        let mut stroke_options = StrokeOptions::default();
        stroke_options.line_width = line_width * 2.0;

        let filled_points: Vec<Position> = {
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

            point_iter.collect()
        };

        // Todo: This is linear and should be optimized
        fn get_closest_point(point: Position, points: &Vec<Position>) -> Position {
            if points.len() > 0 {
                let mut closest = points[0];
                let mut dist = f64::MAX;
                for p in points {
                    let cur_dist = (point.x - p.x).powi(2) + (point.y - p.y).powi(2);
                    if cur_dist < dist {
                        dist = cur_dist;
                        closest = *p;
                    }
                }
                closest
            } else {
                point
            }
        }

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &stroke_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                        let point = vertex.position().to_array();
                        let point = Position::new(point[0] as Scalar, point[1] as Scalar);
                        if vertex.side() == Side::Positive {
                            point
                        } else {
                            let p = point;

                            get_closest_point(p, &filled_points)
                        }
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
