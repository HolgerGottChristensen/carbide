use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::flags::Flags;
use crate::focus::{Focus, Focusable, Refocus};
use crate::layout::{Layout, Layouter};
use crate::render::{Primitive, Render, RenderContext};
use crate::state::{StateSync, ValueCell};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

pub struct Duplicated<T: Widget>(Rc<ValueCell<T>>);

impl Duplicated<Empty> {
    pub fn new<T: Widget>(widget: T) -> Duplicated<T> {
        Duplicated(Rc::new(ValueCell::new(widget)))
    }
}

impl<T: Widget> Duplicated<T> {
    pub fn duplicate(&self) -> Duplicated<T> {
        Duplicated(self.0.clone())
    }
}

impl<T: Widget> CommonWidget for Duplicated<T> {
    fn id(&self) -> WidgetId {
        self.0.borrow().id()
    }

    fn flag(&self) -> Flags {
        self.0.borrow().flag()
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.0.borrow().alignment()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
        self.0.borrow().apply(f, |a, b| a.foreach_child(b))
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_mut(b))
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_rev(b))
    }

    fn position(&self) -> Position {
        self.0.borrow().position()
    }

    fn set_position(&mut self, position: Position) {
        self.0.borrow_mut().set_position(position)
    }

    fn get_focus(&self) -> Focus {
        self.0.borrow().get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.0.borrow_mut().set_focus(focus)
    }

    fn flexibility(&self) -> u32 {
        self.0.borrow().flexibility()
    }

    fn dimension(&self) -> Dimension {
        self.0.borrow().dimension()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.0.borrow_mut().set_dimension(dimension)
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_direct(b))
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_direct_rev(b))

    }
}

impl<T: Widget> StateSync for Duplicated<T> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.0.borrow_mut().capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.0.borrow_mut().release_state(env)
    }
}

impl<T: Widget> MouseEventHandler for Duplicated<T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.0.borrow_mut().handle_mouse_event(event, consumed, env)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.0.borrow_mut().process_mouse_event(event, consumed, env)
    }
}

impl<T: Widget> KeyboardEventHandler for Duplicated<T> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.0.borrow_mut().handle_keyboard_event(event, env)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.0.borrow_mut().process_keyboard_event(event, env)
    }
}

impl<T: Widget> OtherEventHandler for Duplicated<T> {
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.0.borrow_mut().handle_other_event(event, env)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.0.borrow_mut().process_other_event(event, env)
    }
}

impl<T: Widget> Layout for Duplicated<T> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.0.borrow_mut().calculate_size(requested_size, env)
    }

    fn position_children(&mut self, env: &mut Environment) {
        self.0.borrow_mut().position_children(env)
    }
}

impl<T: Widget> Render for Duplicated<T> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.0.borrow_mut().render(context, env)
    }

    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.0.borrow_mut().get_primitives(primitives, env);
    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.0.borrow_mut().process_get_primitives(primitives, env);
    }
}

impl<T: Widget> Focusable for Duplicated<T> {
    fn focus_retrieved(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.0.borrow_mut().focus_retrieved(event, focus_request, env)
    }

    fn focus_dismissed(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.0.borrow_mut().focus_dismissed(event, focus_request, env)
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        self.0.borrow_mut().set_focus_and_request(focus, env)
    }

    fn process_focus_request(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.0.borrow_mut()
            .process_focus_request(event, focus_request, env)
    }

    fn process_focus_next(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.0.borrow_mut()
            .process_focus_next(event, focus_request, focus_up_for_grab, env)
    }

    fn process_focus_previous(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.0.borrow_mut()
            .process_focus_previous(event, focus_request, focus_up_for_grab, env)
    }
}

impl<T: Widget> Debug for Duplicated<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.borrow().fmt(f)
    }
}

impl<T: Widget> Clone for Duplicated<T> {
    fn clone(&self) -> Self {
        Duplicated(self.0.clone())
    }
}

impl<T: Widget> Widget for Duplicated<T> {}

impl<T: Widget> WidgetExt for Duplicated<T> {}