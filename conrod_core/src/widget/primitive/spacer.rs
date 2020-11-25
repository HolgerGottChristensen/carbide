//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.
use {Color, Colorable, Point, Rect, Sizeable, Widget};
use ::{widget, Scalar};
use widget::triangles::Triangle;
use widget::render::Render;
use graph::Container;
use widget::{Id, Rectangle};
use color::rgb;
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use widget::primitive::CWidget;
use position::Dimensions;
use daggy::petgraph::graph::node_index;
use ::{Range, text};
use render::owned_primitive::OwnedPrimitive;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use widget::envelope_editor::EnvelopePoint;
use widget::primitive::shape::triangles::Vertex;
use std::convert::TryFrom;
use std::error::Error;
use std::collections::HashMap;
use std::any::Any;
use widget::layout::Layout;
use text::font::Map;
use layout::basic_layouter::BasicLayouter;


/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Debug)]
pub struct Spacer {
    id: Uuid,
    position: Point,
    dimension: Dimensions
}

impl Spacer {
    pub fn new() -> CWidget {
        CWidget::Spacer(Spacer {
            id: Uuid::new_v4(),
            position: [0.0,0.0],
            dimension: [100.0,100.0]
        })
    }
}

impl Layout for Spacer {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {

    }
}

impl CommonWidget for Spacer {
    fn get_id(&self) -> Uuid {
        self.id
    }
    fn get_children(&self) -> &Vec<CWidget> {
        unimplemented!()
    }

    fn get_position(&self) -> Point {
        unimplemented!()
    }

    fn get_x(&self) -> Scalar {
        self.position[0]
    }

    fn set_x(&mut self, x: f64) {
        self.position = Point::new(x, self.position.get_y());
    }

    fn get_y(&self) -> Scalar {
        self.position[1]
    }

    fn set_y(&mut self, y: f64) {
        self.position = Point::new(self.position.get_x(), y);
    }

    fn get_size(&self) -> Dimensions {
        unimplemented!()
    }

    fn get_width(&self) -> Scalar {
        self.dimension[0]
    }

    fn get_height(&self) -> Scalar {
        self.dimension[1]
    }
}

impl Render for Spacer {
    fn layout(&mut self, proposed_size: Dimensions, fonts: &text::font::Map, positioner: &dyn Fn(&mut CommonWidget, Dimensions)) {
        let dimension = self.dimension.clone();
        positioner(self, dimension);
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        let kind = PrimitiveKind::Rectangle { color: rgb(0.0,1.0, 0.0)};
        return Some(new_primitive(id, kind, clip, container.rect));
    }

    fn get_primitives(&self, proposed_dimensions: Dimensions, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 1.0));
        return prims;
    }
}



