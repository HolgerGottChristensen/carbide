use crate::prelude::*;
use crate::color::Rgba;

use crate::state::global_state::GlobalState;
use crate::widget::Rectangle;
use lyon::tessellation::{VertexBuffers, FillTessellator, FillOptions, BuffersBuilder, FillVertex, StrokeOptions, StrokeTessellator, StrokeVertex, LineCap, LineJoin};
use crate::widget::types::triangle_store::TriangleStore;
use crate::widget::primitive::canvas::context::Context;
use crate::render::primitive_kind::PrimitiveKind;

use crate::draw::shape::triangle::Triangle;
use crate::state::environment_color::EnvironmentColor;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Canvas<GS> where GS: GlobalState {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    #[state] color: ColorState<GS>,
    triangle_store: TriangleStore,
    context: Context
}

impl<GS: GlobalState> WidgetExt<GS> for Canvas<GS> {}

impl<S: GlobalState> Layout<S> for Canvas<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, _: &Environment<S>) -> Dimensions {
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {

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

impl<S: GlobalState> Render<S> for Canvas<S> {

    fn get_primitives(&mut self, _: &text::font::Map) -> Vec<Primitive> {

        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();

        let mut tessellator = StrokeTessellator::new();

        for path in self.context.to_paths(self.position) {
            tessellator.tessellate_path(
                &path,
                &StrokeOptions::default().with_line_width(2.0).with_line_cap(LineCap::Round).with_line_join(LineJoin::Round),
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    let point = vertex.position().to_array();
                    [point[0] as Scalar, point[1] as Scalar]
                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Point> = point_iter.collect();


        //self.triangle_store.position = self.position;
        //self.triangle_store.dimensions = self.dimension;
        //self.triangle_store.set_triangles(&triangles);


        let mut prims = vec![
            Primitive {
                kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(*self.color.get_latest_value()), triangles: Triangle::from_point_list(points)},
                rect: Rect::new(self.position, self.dimension)
            }
        ];

        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl<S: GlobalState> Canvas<S> {

    pub fn color(mut self, color: ColorState<S>) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    pub fn initialize(context: Context) -> Box<Self> {
        Box::new(Canvas {
            id: Uuid::new_v4(),
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            color: EnvironmentColor::Accent.into(),
            triangle_store: TriangleStore::new(),
            context
        })
    }
}
