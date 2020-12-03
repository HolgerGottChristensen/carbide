//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.
use {Color, Colorable, Point, Rect, Sizeable, OldWidget};
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
use widget::primitive::Widget;
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
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use state::state::{StateList, DefaultState};
use flags::Flags;
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

impl Event for Spacer {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        ()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList<DefaultState>) -> StateList<DefaultState> {
        state
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList<DefaultState>) -> StateList<DefaultState> {
        state
    }

    fn get_state(&self, current_state: StateList<DefaultState>) -> StateList<DefaultState> {
        unimplemented!()
    }

    fn apply_state(&mut self, states: StateList<DefaultState>) -> StateList<DefaultState> {
        unimplemented!()
    }

    fn sync_state(&mut self, states: StateList<DefaultState>) {
        ()
    }
}

impl Layout for Spacer {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
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

impl CommonWidget for Spacer {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Spacer
    }

    fn get_children(&self) -> WidgetIter {
        unimplemented!()
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
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

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 1.0));
        return prims;
    }
}



