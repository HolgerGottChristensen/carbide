use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;
use uuid::Uuid;

use crate::{Scalar, text};
use crate::event::event::Event;
use crate::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::flags::Flags;
use crate::layout::Layout;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::state::environment::Environment;
use crate::state::state_sync::StateSync;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::edge_insets::EdgeInsets;
use crate::widget::primitive::frame::Frame;
use crate::widget::primitive::padding::Padding;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::widget::primitive::clip::Clip;
use crate::widget::primitive::hidden::Hidden;

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
    fn clip(self) -> Box<Clip<S>> {
        Clip::new(Box::new(self))
    }

    fn hidden(self) -> Box<Hidden<S>> {
        Hidden::new(Box::new(self))
    }
}

//This does not currently work with intellisense
//impl<T> WidgetExt for T where T: Widget + 'static {}

impl<S> CommonWidget<S> for Box<dyn Widget<S>> {
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


impl<S> Event<S> for Box<dyn Widget<S>> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        self.deref_mut().handle_mouse_event(event, consumed, global_state)
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        self.deref_mut().handle_keyboard_event(event, global_state)
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        self.deref_mut().handle_other_event(event)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().process_mouse_event(event, consumed, env, global_state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().process_keyboard_event(event, env, global_state)
    }
}

impl<S> StateSync<S> for Box<dyn Widget<S>> {
    fn insert_local_state(&self, env: &mut Environment<S>) {
        self.deref().insert_local_state(env)
    }

    fn update_all_widget_state(&mut self, env: &Environment<S>, global_state: &S) {
        self.deref_mut().update_all_widget_state(env, global_state)
    }

    fn update_local_widget_state(&mut self, env: &Environment<S>) {
        self.deref_mut().update_local_widget_state(env)
    }

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        self.deref_mut().sync_state(env, global_state)
    }
}

impl<S> Layout<S> for Box<dyn Widget<S>> {
    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], env: &Environment<S>) -> [f64; 2] {
        self.deref_mut().calculate_size(requested_size, env)
    }

    fn position_children(&mut self) {
        self.deref_mut().position_children()
    }
}

impl<S> Render<S> for Box<dyn Widget<S>> {
    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        self.deref().get_primitives(fonts)
    }
}


impl<S> Debug for dyn Widget<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.get_id())
    }
}