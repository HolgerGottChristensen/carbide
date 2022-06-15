use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;

use crate::draw::{Dimension, Position};
use crate::event::{
    Event, KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler,
    WidgetEvent,
};
use crate::focus::{Focus, Focusable, Refocus};
use crate::prelude::*;

pub trait Widget: Event + Layout + Render + Focusable + DynClone + Debug {}

//impl<S, T> Widget<S> for T where T: Event<S> + Layout<S> + Render<S> + DynClone {}

impl Widget for Box<dyn Widget> {}
impl WidgetExt for Box<dyn Widget> {}

dyn_clone::clone_trait_object!(Widget);

//This does not currently work with intellisense
//impl<T> WidgetExt for T where T: Widget + 'static {}

impl<T: Widget + ?Sized> CommonWidget for Box<T> {
    fn id(&self) -> WidgetId {
        self.deref().id()
    }

    fn set_id(&mut self, id: WidgetId) {
        self.deref_mut().set_id(id);
    }

    fn flag(&self) -> Flags {
        self.deref().flag()
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.deref().alignment()
    }

    fn children(&self) -> WidgetIter {
        self.deref().children()
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        self.deref_mut().children_mut()
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        self.deref_mut().children_direct()
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        self.deref_mut().children_direct_rev()
    }

    fn position(&self) -> Position {
        self.deref().position()
    }

    fn set_position(&mut self, position: Position) {
        self.deref_mut().set_position(position)
    }

    fn get_focus(&self) -> Focus {
        self.deref().get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.deref_mut().set_focus(focus)
    }

    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn dimension(&self) -> Dimension {
        self.deref().dimension()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.deref_mut().set_dimension(dimension)
    }
}

impl<T: Widget + ?Sized> MouseEventHandler for Box<T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.deref_mut().handle_mouse_event(event, consumed, env)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.deref_mut().process_mouse_event(event, consumed, env)
    }
}

impl<T: Widget + ?Sized> KeyboardEventHandler for Box<T> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.deref_mut().handle_keyboard_event(event, env)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.deref_mut().process_keyboard_event(event, env)
    }
}

impl<T: Widget + ?Sized> OtherEventHandler for Box<T> {
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.deref_mut().handle_other_event(event, env)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.deref_mut().process_other_event(event, env)
    }
}

impl<T: Widget + ?Sized> StateSync for Box<T> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.deref_mut().capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.deref_mut().release_state(env)
    }
}

impl<T: Widget + ?Sized> Layout for Box<T> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.deref_mut().calculate_size(requested_size, env)
    }

    fn position_children(&mut self) {
        self.deref_mut().position_children()
    }
}

impl<T: Widget + ?Sized> Render for Box<T> {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.deref_mut().get_primitives(primitives, env);
    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.deref_mut().process_get_primitives(primitives, env);
    }
}

impl<T: Widget + ?Sized> Focusable for Box<T> {
    fn focus_retrieved(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.deref_mut().focus_retrieved(event, focus_request, env)
    }

    fn focus_dismissed(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.deref_mut().focus_dismissed(event, focus_request, env)
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        self.deref_mut().set_focus_and_request(focus, env)
    }

    fn process_focus_request(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.deref_mut()
            .process_focus_request(event, focus_request, env)
    }

    fn process_focus_next(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.deref_mut()
            .process_focus_next(event, focus_request, focus_up_for_grab, env)
    }

    fn process_focus_previous(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.deref_mut()
            .process_focus_previous(event, focus_request, focus_up_for_grab, env)
    }
}

/*impl Debug for dyn Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.id())
    }
}*/
