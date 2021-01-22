//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.

use daggy::petgraph::graph::node_index;
use uuid::Uuid;

use crate::{Color, Colorable, Point, Rect, Sizeable};
use crate::{Scalar, widget};
use crate::text;
use crate::draw::shape::triangle::Triangle;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;
use crate::state::environment::Environment;
use crate::state::state_sync::NoLocalStateSync;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::color::Rgba;

use crate::state::global_state::GlobalState;
use crate::widget::Rectangle;
use lyon::tessellation::{VertexBuffers, FillTessellator, FillOptions, BuffersBuilder, FillVertex};
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::path::{Path, Winding};
use lyon::tessellation::path::traits::PathBuilder;
use lyon::tessellation::math::rect;
use lyon::tessellation::path::builder::BorderRadii;
use crate::widget::types::triangle_store::TriangleStore;
use crate::widget::primitive::canvas::context::Context;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Canvas {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    color: Color,
    triangle_store: TriangleStore,
    context: Context
}

impl<GS: GlobalState> WidgetExt<GS> for Canvas {}

impl<S: GlobalState> Layout<S> for Canvas {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {

    }
}

impl<S: GlobalState> CommonWidget<S> for Canvas {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
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

impl<S: GlobalState> Render<S> for Canvas {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {

        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();

        let mut tessellator = FillTessellator::new();

        for path in self.context.to_paths(self.position) {
            tessellator.tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
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
                id: node_index(0),
                kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(self.color), triangles: Triangle::from_point_list(points)},
                scizzor: Rect::new(self.position, self.dimension),
                rect: Rect::new(self.position, self.dimension)
            }
        ];

        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl Canvas {

    pub fn fill(mut self, color: Color) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    pub fn initialize(context: Context) -> Box<Canvas> {
        Box::new(Canvas {
            id: Uuid::new_v4(),
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            color: Color::random(),
            triangle_store: TriangleStore::new(),
            context
        })
    }
}
