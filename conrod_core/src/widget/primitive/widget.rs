use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::slice::{Iter, IterMut};

use dyn_clone::DynClone;
use uuid::Uuid;

use ::{Color, Rect};
use ::{Scalar, text};
use color::rgb;
use event::event::Event;
use event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use flags::Flags;
use graph::Container;
use layout::Layout;
use position::Dimensions;
use render::owned_primitive::OwnedPrimitive;
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use state::environment::Environment;
use state::state::LocalStateList;
use state::state_sync::StateSync;
use text::font::Map;
use widget::{common_widget, Id, Image, Line, Oval, Text};
use widget::common_widget::CommonWidget;
use widget::primitive::edge_insets::EdgeInsets;
use widget::primitive::frame::Frame;
use widget::primitive::h_stack::HStack;
use widget::primitive::padding::Padding;
use widget::primitive::shape::oval::Full;
use widget::primitive::spacer::Spacer;
use widget::primitive::v_stack::VStack;
use widget::primitive::z_stack::ZStack;
use widget::render::Render;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

use crate::widget::primitive::shape::rectangle::Rectangle;

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

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: LocalStateList, global_state: &mut S) -> LocalStateList {
        self.deref_mut().process_mouse_event(event, consumed, state, global_state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: LocalStateList, global_state: &mut S) -> LocalStateList {
        self.deref_mut().process_keyboard_event(event, state, global_state)
    }
}

impl<S> StateSync<S> for Box<Widget<S>> {
    fn push_local_state(&self, env: &mut Environment) {
        self.deref().push_local_state(env);
    }

    fn pop_local_state(&self, env: &mut Environment) {
        self.deref().pop_local_state(env);
    }

    fn replace_local_state(&self, env: &mut Environment) {
        self.deref().replace_local_state(env)
    }

    fn update_widget_state(&mut self, env: &Environment, global_state: &S) {
        self.deref_mut().update_widget_state(env, global_state)
    }

    fn sync_state(&mut self, env: &mut Environment, global_state: &S) {
        self.deref_mut().sync_state(env, global_state)
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