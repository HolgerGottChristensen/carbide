use crate::widget::primitive::shape::rectangle::Rectangle;
use ::{Color, Rect};
use color::rgb;
use graph::Container;
use widget::{Id, Oval, Line, Text, Image, common_widget};
use widget::render::Render;
use widget::primitive::shape::oval::Full;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use render::primitive::Primitive;
use render::owned_primitive::OwnedPrimitive;
use ::{text, Scalar};
use position::Dimensions;
use widget::common_widget::CommonWidget;
use widget::primitive::v_stack::VStack;
use uuid::Uuid;

use text::font::Map;
use widget::primitive::frame::Frame;
use widget::primitive::h_stack::HStack;
use widget::primitive::z_stack::ZStack;
use widget::primitive::spacer::Spacer;
use widget::primitive::edge_insets::EdgeInsets;
use widget::primitive::padding::Padding;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use std::fmt::Debug;
use std::fmt;
use std::ops::{Deref, DerefMut};
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIterMut, WidgetIter};
use std::slice::{Iter, IterMut};
use dyn_clone::DynClone;
use layout::Layout;
use state::environment::Environment;

pub trait Widget<S>: Event<S> + Layout<S> + Render<S> + DynClone {}

impl<S, T> Widget<S> for T where T: Event<S> + Layout<S> + Render<S> + DynClone {}

dyn_clone::clone_trait_object!(<S> Widget<S>);

pub trait WidgetExt<S: 'static>: Widget<S> + Sized + 'static {
    fn frame(self, width: Scalar, height: Scalar) -> Box<Frame<S>> {
        Frame::init(width, height, Box::new(self))
    }

    fn padding(self, edge_insets: EdgeInsets) -> Box<Padding<S>> {
        Padding::init(edge_insets, Box::new(self))
    }
}

//This does not currently work with intellisense
//impl<T> WidgetExt for T where T: Widget + 'static {}

impl<S> CommonWidget<S> for Box<Widget<S>> {
    fn get_id(&self) -> Uuid {
        self.deref().get_id()
    }

    fn get_flag(&self) -> Flags {
        self.deref().get_flag()
    }

    fn get_children(&self) -> WidgetIter<S> {
        self.deref().get_children()
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        self.deref_mut().get_children_mut()
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        self.deref_mut().get_proxied_children()
    }

    fn get_position(&self) -> Dimensions {
        self.deref().get_position()
    }

    fn set_position(&mut self, position: Dimensions) {
        self.deref_mut().set_position(position)
    }

    fn get_dimension(&self) -> Dimensions {
        self.deref().get_dimension()
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.deref_mut().set_dimension(dimensions)
    }
}



impl<S> Event<S> for Box<Widget<S>> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        self.deref_mut().handle_mouse_event(event, consumed, global_state)
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        self.deref_mut().handle_keyboard_event(event, global_state)
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        self.deref_mut().handle_other_event(event)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList, global_state: &mut S) -> StateList {
        self.deref_mut().process_mouse_event(event, consumed, state, global_state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        self.deref_mut().process_keyboard_event(event, state, global_state)
    }

    fn get_state(&self, mut current_state: StateList) -> StateList {
        self.deref().get_state(current_state)
    }

    fn apply_state(&mut self, mut states: StateList, global_state: &S) -> StateList {
        self.deref_mut().apply_state(states, global_state)
    }

    fn sync_state(&mut self, mut states: StateList, global_state: &S) {
        self.deref_mut().sync_state(states, global_state)
    }
}

impl<S> Layout<S> for Box<Widget<S>> {
    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], env: &Environment) -> [f64; 2] {
        self.deref_mut().calculate_size(requested_size, env)
    }

    fn position_children(&mut self) {
        self.deref_mut().position_children()
    }
}

impl<S> Render<S> for Box<Widget<S>> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        self.deref().get_primitives(fonts)
    }
}


impl<S> Debug for Widget<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.get_id())
    }
}