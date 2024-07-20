use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;

use crate::draw::{Alignment, Dimension, Position};
use crate::environment::Environment;
use crate::event::{Event, EventHandler, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::flags::WidgetFlag;
use crate::focus::{Focus, Focusable, FocusContext};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::state::StateSync;
use crate::update::{Update, UpdateContext};
use crate::widget::{CommonWidget, WidgetExt, WidgetId, WidgetSync};

// TODO Rename to AnyWidget and create a widget that is anywidget and clone
pub trait AnyWidget: EventHandler + Update + Layout + Render + Focusable + DynClone + Debug + 'static {}

dyn_clone::clone_trait_object!(AnyWidget);


pub trait Widget: AnyWidget + WidgetExt + Clone + private::Sealed {}

impl<T> Widget for T where T: AnyWidget + WidgetExt + Clone {}

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

    fn alignment(&self) -> Alignment {
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

impl<T: AnyWidget + ?Sized> WidgetSync for Box<T> {
    fn sync(&mut self, env: &mut Environment) {
        self.deref_mut().sync(env);
    }
}

impl<T: AnyWidget + ?Sized> Update for Box<T> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.deref_mut().update(ctx);
    }

    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.deref_mut().process_update(ctx);
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
    fn render(&mut self, context: &mut RenderContext) {
        self.deref_mut().render(context)
    }
}

impl<T: AnyWidget + ?Sized> Focusable for Box<T> {
    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        self.deref_mut().process_focus_request(ctx)
    }

    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        self.deref_mut().process_focus_next(ctx)
    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {
        self.deref_mut().process_focus_previous(ctx)
    }
}

