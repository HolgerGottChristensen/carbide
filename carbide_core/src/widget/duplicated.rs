use std::cell::{Ref, RefCell};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::event::Event;
use crate::flags::WidgetFlag;
use crate::focus::{Focus, Focusable, Refocus};
use crate::layout::{Layout, LayoutContext, Layouter};
use crate::render::{Render, RenderContext};
use crate::state::StateSync;
use crate::update::{Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Empty, WidgetExt, WidgetId};

pub struct Duplicated<T: AnyWidget>(Rc<RefCell<T>>);

impl Duplicated<Empty> {
    pub fn new<T: AnyWidget>(widget: T) -> Duplicated<T> {
        Duplicated(Rc::new(RefCell::new(widget)))
    }
}

impl<T: AnyWidget> Duplicated<T> {
    pub fn duplicate(&self) -> Duplicated<T> {
        Duplicated(self.0.clone())
    }
}

impl<T: AnyWidget> CommonWidget for Duplicated<T> {
    fn id(&self) -> WidgetId {
        self.0.borrow().id()
    }

    fn flag(&self) -> WidgetFlag {
        self.0.borrow().flag()
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.0.borrow().alignment()
    }

    fn foreach_child<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        todo!()//self.0.borrow().apply(f, |a, b| a.foreach_child(b))
    }

    fn foreach_child_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        todo!()//self.0.borrow_mut().apply(f, |a, b| a.foreach_child_mut(b))
    }

    fn foreach_child_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        todo!()//self.0.borrow_mut().apply(f, |a, b| a.foreach_child_rev(b))
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

    fn foreach_child_direct<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        todo!()//self.0.borrow_mut().apply(f, |a, b| a.foreach_child_direct(b))
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        todo!()//self.0.borrow_mut().apply(f, |a, b| a.foreach_child_direct_rev(b))
    }
}

impl<T: AnyWidget> StateSync for Duplicated<T> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.0.borrow_mut().capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.0.borrow_mut().release_state(env)
    }
}

impl<T: AnyWidget> Update for Duplicated<T> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.0.borrow_mut().update(ctx)
    }

    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.0.borrow_mut().process_update(ctx)
    }
}

impl<T: AnyWidget> MouseEventHandler for Duplicated<T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.0.borrow_mut().handle_mouse_event(event, ctx)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.0.borrow_mut().process_mouse_event(event, ctx)
    }
}

impl<T: AnyWidget> KeyboardEventHandler for Duplicated<T> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.0.borrow_mut().handle_keyboard_event(event, ctx)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.0.borrow_mut().process_keyboard_event(event, ctx)
    }
}

impl<T: AnyWidget> WindowEventHandler for Duplicated<T> {
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.0.borrow_mut().handle_window_event(event, ctx)
    }

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.0.borrow_mut().process_window_event(event, ctx)
    }
}

impl<T: AnyWidget> OtherEventHandler for Duplicated<T> {
    fn handle_other_event(&mut self, _event: &Event, ctx: &mut OtherEventContext) {
        self.0.borrow_mut().handle_other_event(_event, ctx)
    }

    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        self.0.borrow_mut().process_other_event(event, ctx)
    }
}

impl<T: AnyWidget> Layout for Duplicated<T> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.0.borrow_mut().calculate_size(requested_size, ctx)
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.0.borrow_mut().position_children(ctx)
    }
}

impl<T: AnyWidget> Render for Duplicated<T> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.0.borrow_mut().render(context, env)
    }
}

impl<T: AnyWidget> Focusable for Duplicated<T> {
    fn focus_retrieved(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.0.borrow_mut().focus_retrieved(focus_request, env)
    }

    fn focus_dismissed(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.0.borrow_mut().focus_dismissed(focus_request, env)
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        self.0.borrow_mut().set_focus_and_request(focus, env)
    }

    fn process_focus_request(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.0.borrow_mut()
            .process_focus_request(focus_request, env)
    }

    fn process_focus_next(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.0.borrow_mut()
            .process_focus_next(focus_request, focus_up_for_grab, env)
    }

    fn process_focus_previous(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.0.borrow_mut()
            .process_focus_previous(focus_request, focus_up_for_grab, env)
    }
}

impl<T: AnyWidget> Debug for Duplicated<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.borrow().fmt(f)
    }
}

impl<T: AnyWidget> Clone for Duplicated<T> {
    fn clone(&self) -> Self {
        Duplicated(self.0.clone())
    }
}

impl<T: AnyWidget> AnyWidget for Duplicated<T> {}

impl<T: AnyWidget> WidgetExt for Duplicated<T> {}