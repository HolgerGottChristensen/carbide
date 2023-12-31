use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;

use carbide_core::render::RenderContext;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{EventHandler, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::event::Event;
use crate::flags::WidgetFlag;
use crate::focus::{Focus, Focusable, Refocus};
use crate::layout::{Layout, LayoutContext, Layouter};
use crate::render::Render;
use crate::state::StateSync;
use crate::widget::{CommonWidget, WidgetExt, WidgetId};

// TODO Rename to AnyWidget and create a widget that is anywidget and clone
pub trait AnyWidget: EventHandler + Layout + Render + Focusable + DynClone + Debug + 'static {}

dyn_clone::clone_trait_object!(AnyWidget);


pub trait Widget: AnyWidget + Clone + private::Sealed {}

impl<T> Widget for T where T: AnyWidget + Clone {}

mod private {
    use crate::widget::AnyWidget;

    // This disallows implementing Widget manually, and requires something to implement
    // AnyWidget to implement Widget.
    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyWidget {}
}

// ---------------------------------------------------
//  Implement Widget for Box dyn Widget
// ---------------------------------------------------

impl AnyWidget for Box<dyn AnyWidget> {}

impl WidgetExt for Box<dyn AnyWidget> {}

impl<T: AnyWidget + ?Sized> CommonWidget for Box<T> {
    fn id(&self) -> WidgetId {
        self.deref().id()
    }

    fn flag(&self) -> WidgetFlag {
        self.deref().flag()
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.deref().alignment()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        self.deref().foreach_child(f)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_mut(f)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_rev(f)
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

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_direct(f)
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_direct_rev(f)
    }
}

impl<T: AnyWidget + ?Sized> MouseEventHandler for Box<T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.deref_mut().handle_mouse_event(event, ctx)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.deref_mut().process_mouse_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> KeyboardEventHandler for Box<T> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.deref_mut().handle_keyboard_event(event, ctx)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.deref_mut().process_keyboard_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> WindowEventHandler for Box<T> {
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.deref_mut().handle_window_event(event, ctx)
    }

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.deref_mut().process_window_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> OtherEventHandler for Box<T> {
    fn handle_other_event(&mut self, _event: &Event, ctx: &mut OtherEventContext) {
        self.deref_mut().handle_other_event(_event, ctx)
    }

    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        self.deref_mut().process_other_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> StateSync for Box<T> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.deref_mut().capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.deref_mut().release_state(env)
    }
}

impl<T: AnyWidget + ?Sized> Layout for Box<T> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.deref_mut().calculate_size(requested_size, ctx)
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.deref_mut().position_children(ctx)
    }
}

impl<T: AnyWidget + ?Sized> Render for Box<T> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.deref_mut().render(context, env)
    }
}

impl<T: AnyWidget + ?Sized> Focusable for Box<T> {
    fn focus_retrieved(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.deref_mut().focus_retrieved(focus_request, env)
    }

    fn focus_dismissed(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.deref_mut().focus_dismissed(focus_request, env)
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        self.deref_mut().set_focus_and_request(focus, env)
    }

    fn process_focus_request(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.deref_mut()
            .process_focus_request(focus_request, env)
    }

    fn process_focus_next(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.deref_mut()
            .process_focus_next(focus_request, focus_up_for_grab, env)
    }

    fn process_focus_previous(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.deref_mut()
            .process_focus_previous(focus_request, focus_up_for_grab, env)
    }
}

