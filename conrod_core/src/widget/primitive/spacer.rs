//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.
use std::any::Any;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;

use daggy::petgraph::graph::node_index;
use uuid::Uuid;

use {Color, Colorable, Point, Rect, Sizeable};
use ::{Scalar, widget};
use ::{Range, text};
use color::rgb;
use event::event::{Event, NoEvents};
use event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use flags::Flags;
use graph::Container;
use layout::basic_layouter::BasicLayouter;
use layout::Layout;
use position::Dimensions;
use render::owned_primitive::OwnedPrimitive;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use state::environment::Environment;
use state::state::LocalStateList;
use state::state_sync::NoLocalStateSync;
use text::font::Map;
use widget::{Id, Rectangle};
use widget::common_widget::CommonWidget;
use widget::primitive::Widget;
use widget::render::Render;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Debug)]
pub struct Spacer {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    space: SpacerDirection
}

#[derive(Clone, Debug)]
pub enum SpacerDirection {
    Vertical,
    Horizontal,
    Both
}



impl Spacer {
    pub fn new(space: SpacerDirection) -> Box<Self> {
        Box::new(Spacer {
            id: Uuid::new_v4(),
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            space
        })
    }
}

impl NoEvents for Spacer {}

impl NoLocalStateSync for Spacer {}

impl<S> Layout<S> for Spacer {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment) -> Dimensions {
        match self.space {
            SpacerDirection::Vertical => {
                self.dimension = [0.0, requested_size[1]];
            }
            SpacerDirection::Horizontal => {
                self.dimension = [requested_size[0], 0.0];
            }
            SpacerDirection::Both => {
                self.dimension = requested_size;
            }
        }

        self.dimension
    }

    fn position_children(&mut self) {

    }
}

impl<S> CommonWidget<S> for Spacer {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Spacer
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

impl<S> Render<S> for Spacer {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::rect_outline(Rect::new(self.position, self.dimension), 1.0));
        return prims;
    }
}



