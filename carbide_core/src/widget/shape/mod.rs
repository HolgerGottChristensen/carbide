use lyon::lyon_tessellation::path::path::Builder;
use lyon::lyon_tessellation::StrokeAlignment;
use lyon::math::Box2D;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, Side, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
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
use crate::draw::shape::triangle::Triangle;
use crate::environment::Environment;
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
    // Todo: add primitives to before and after the shape.
    fn triangles(&mut self, env: &mut Environment) -> Vec<Triangle<Position>> {
        let mut geom = Tris(vec![]);
        self.render(&mut RenderContext {
                    render: &mut geom,
                    text: &mut NOOPTextContext,
                    image: &mut NOOPImageContext,
            env,
        });

        geom.0
    }
}

dyn_clone::clone_trait_object!(Shape);

impl AnyWidget for Box<dyn Shape> {}


struct Tris(Vec<Triangle<Position>>);
impl InnerRenderContext for Tris {
    fn transform(&mut self, _transform: CarbideTransform) {}

    fn pop_transform(&mut self) {}

    fn color_filter(&mut self, _hue_rotation: f32, _saturation_shift: f32, _luminance_shift: f32, _color_invert: bool) {}

    fn pop_color_filter(&mut self) {}

    fn clip(&mut self, _bounding_box: Rect) {}

    fn pop_clip(&mut self) {}

    fn filter(&mut self, _id: FilterId, _bounding_box: Rect) {}

    fn filter2d(&mut self, _id1: FilterId, _bounding_box1: Rect, _id2: FilterId, _bounding_box2: Rect) {}

    fn stencil(&mut self, _geometry: &[Triangle<Position>]) {}

    fn pop_stencil(&mut self) {}

    fn geometry(&mut self, geometry: &[Triangle<Position>]) {
        self.0.extend(geometry);
    }

    fn stroke(&mut self, _stroke: &[Triangle<(Position, (Position, Position, f32, f32))>]) {}

    fn style(&mut self, _style: DrawStyle) {}

    fn pop_style(&mut self) {}

    fn stroke_dash_pattern(&mut self, _pattern: Option<StrokeDashPattern>) {}

    fn pop_stroke_dash_pattern(&mut self) {}

    fn image(&mut self, _id: Option<ImageId>, _bounding_box: Rect, _source_rect: Rect, _mode: u32) {}

    fn text(&mut self, _text: TextId, _ctx: &mut dyn InnerTextContext) {}

    fn filter_new(&mut self) {}

    fn filter_new_pop(&mut self, _id: FilterId, _color: Color, _post_draw: bool) {}

    fn filter_new_pop2d(&mut self, _id: FilterId, _id2: FilterId, _color: Color, _post_draw: bool) {}

    fn mask_start(&mut self) {}

    fn mask_in(&mut self) {}

    fn mask_end(&mut self) {}

    fn layer(&mut self, layer_id: LayerId, dimensions: Dimension) -> Layer {
        static LAYER: NoopLayer = NoopLayer;
        Layer {
            inner: &LAYER,
            inner2: &LAYER,
        }
    }

    fn render_layer(&mut self, layer_id: LayerId, bounding_box: Rect) {}
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
