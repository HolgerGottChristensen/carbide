use core::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;

use crate::draw::{Dimension, Position};
use crate::event::{Event, KeyboardEvent, MouseEvent, WidgetEvent};
use crate::focus::{Focus, Focusable, Refocus};
use crate::prelude::*;
use crate::render::RenderProcessor;
use crate::widget::{Frame, Offset};
use crate::widget::Border;
use crate::widget::Clip;
use crate::widget::EdgeInsets;
use crate::widget::Hidden;
use crate::widget::Padding;

pub trait Widget: Event + Layout + Render + RenderProcessor + Focusable + DynClone {}

//impl<S, T> Widget<S> for T where T: Event<S> + Layout<S> + Render<S> + DynClone {}

impl Widget for Box<dyn Widget> {}

dyn_clone::clone_trait_object!(Widget);

//This does not currently work with intellisense
//impl<T> WidgetExt for T where T: Widget + 'static {}

impl CommonWidget for Box<dyn Widget> {
    fn get_id(&self) -> Uuid {
        self.deref().get_id()
    }

    fn set_id(&mut self, id: Uuid) {
        self.deref_mut().set_id(id);
    }

    fn get_flag(&self) -> Flags {
        self.deref().get_flag()
    }

    fn get_children(&self) -> WidgetIter {
        self.deref().get_children()
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        self.deref_mut().get_children_mut()
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        self.deref_mut().get_proxied_children()
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        self.deref_mut().get_proxied_children_rev()
    }

    fn get_position(&self) -> Position {
        self.deref().get_position()
    }

    fn set_position(&mut self, position: Position) {
        self.deref_mut().set_position(position)
    }

    fn get_dimension(&self) -> Dimension {
        self.deref().get_dimension()
    }

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.deref_mut().set_dimension(dimensions)
    }
}


impl Event for Box<dyn Widget> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.deref_mut().handle_mouse_event(event, consumed, env)
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.deref_mut().handle_keyboard_event(event, env)
    }

    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.deref_mut().handle_other_event(event, env)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.deref_mut().process_mouse_event(event, consumed, env)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.deref_mut().process_keyboard_event(event, env)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.deref_mut().process_other_event(event, env)
    }
}

impl StateSync for Box<dyn Widget> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.deref_mut().capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.deref_mut().release_state(env)
    }
}

impl Layout for Box<dyn Widget> {
    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.deref_mut().calculate_size(requested_size, env)
    }

    fn position_children(&mut self) {
        self.deref_mut().position_children()
    }
}

impl RenderProcessor for Box<dyn Widget> {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.deref_mut().process_get_primitives(primitives, env);
    }
}

impl Render for Box<dyn Widget> {
    fn get_primitives(&mut self, env: &mut Environment) -> Vec<Primitive> {
        self.deref_mut().get_primitives(env)
    }
}

impl Focusable for Box<dyn Widget> {
    fn focus_retrieved(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment) {
        self.deref_mut().focus_retrieved(event, focus_request, env)
    }

    fn focus_dismissed(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment) {
        self.deref_mut().focus_dismissed(event, focus_request, env)
    }

    fn get_focus(&self) -> Focus {
        self.deref().get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.deref_mut().set_focus(focus)
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        self.deref_mut().set_focus_and_request(focus, env)
    }

    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment) -> bool {
        self.deref_mut().process_focus_request(event, focus_request, env)
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        self.deref_mut().process_focus_next(event, focus_request, focus_up_for_grab, env)
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        self.deref_mut().process_focus_previous(event, focus_request, focus_up_for_grab, env)
    }
}


impl Debug for dyn Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.get_id())
    }
}