use lyon::algorithms::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers};

use crate::color::Rgba;
use crate::draw::shape::triangle::Triangle;
use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
use crate::state::global_state::GlobalState;
use crate::widget::primitive::canvas::context::{Context, ShapeStyleWithOptions};
use crate::widget::Rectangle;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Canvas<GS> where GS: GlobalState {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    #[state] color: ColorState<GS>,
    //prim_store: Vec<Primitive>,
    context: fn(Rect, Context<GS>) -> Context<GS>,
}

impl<GS: GlobalState> Canvas<GS> {
    pub fn initialize(context: fn(Rect, Context<GS>) -> Context<GS>) -> Box<Self> {
        Box::new(Canvas {
            id: Uuid::new_v4(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            color: EnvironmentColor::Accent.into(),
            //prim_store: vec![],
            context,
        })
    }

    pub fn get_stroke_prim(&self, path: Path, stroke_options: StrokeOptions, color: Color) -> Primitive {
        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();

        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    let point = vertex.position().to_array();
                    [point[0] as Scalar, point[1] as Scalar]
                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Point> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(color), triangles: Triangle::from_point_list(points) },
            rect: Rect::new(self.position, self.dimension),
        }
    }

    pub fn get_fill_prim(&self, path: Path, fill_options: FillOptions, color: Color) -> Primitive {
        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &fill_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    let point = vertex.position().to_array();
                    [point[0] as Scalar, point[1] as Scalar]
                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Point> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(color), triangles: Triangle::from_point_list(points) },
            rect: Rect::new(self.position, self.dimension),
        }
    }
}

impl<S: GlobalState> CommonWidget<S> for Canvas<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<GS: GlobalState> Render<GS> for Canvas<GS> {
    fn get_primitives(&mut self, env: &Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        let context = Context::new();

        let rectangle = Rect::new(self.get_position(), self.get_dimension());
        let context = (self.context)(rectangle, context);

        let paths = context.to_paths(self.get_position());

        let mut prims = vec![];

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, color) => {
                    prims.push(self.get_fill_prim(path, fill_options, *color.clone().get_value(env, global_state)));
                }
                ShapeStyleWithOptions::Stroke(stroke_options, color) => {
                    prims.push(self.get_stroke_prim(path, stroke_options, *color.clone().get_value(env, global_state)));
                }
            }
        }

        prims.extend(Rectangle::<GS>::debug_outline(Rect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl<GS: GlobalState> WidgetExt<GS> for Canvas<GS> {}

impl<GS: GlobalState> Layout<GS> for Canvas<GS> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {}
}