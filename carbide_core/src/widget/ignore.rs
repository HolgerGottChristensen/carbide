use std::fmt::Debug;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::event::Event;
use crate::flags::WidgetFlag;
use crate::focus::{Focus, Focusable, Refocus};
use crate::layout::{Layout, LayoutContext, Layouter};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState, StateSync};
use crate::update::{Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Clone, Debug)]
pub struct Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> where
    T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
{
    inner: T,
    state_sync: B1,
    mouse_event: B2,
    keyboard_event: B3,
    other_event: B4,
    layout_event: B5,
    render_event: B6,
    focus_event: B7,
    update: B8,
}

impl Ignore<Empty, bool, bool, bool, bool, bool, bool, bool, bool> {
    pub fn new<T: Widget>(widget: T) -> Ignore<T, bool, bool, bool, bool, bool, bool, bool, bool> {
        Ignore {
            inner: widget,
            state_sync: true,
            mouse_event: true,
            keyboard_event: true,
            other_event: true,
            layout_event: true,
            render_event: true,
            focus_event: true,
            update: true,
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    pub fn render<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, B4, B5, V::Output, B7, B8> {
        Ignore {
            inner: self.inner,
            state_sync: self.state_sync,
            mouse_event: self.mouse_event,
            keyboard_event: self.keyboard_event,
            other_event: self.other_event,
            layout_event: self.layout_event,
            render_event: value.into_read_state(),
            focus_event: self.focus_event,
            update: self.update,
        }
    }

    pub fn layout<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, B4, V::Output, B6, B7, B8> {
        Ignore {
            inner: self.inner,
            state_sync: self.state_sync,
            mouse_event: self.mouse_event,
            keyboard_event: self.keyboard_event,
            other_event: self.other_event,
            layout_event: value.into_read_state(),
            render_event: self.render_event,
            focus_event: self.focus_event,
            update: self.update,
        }
    }

    pub fn accept_mouse_events<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, V::Output, B3, B4, B5, B6, B7, B8> {
        Ignore {
            inner: self.inner,
            state_sync: self.state_sync,
            mouse_event: value.into_read_state(),
            keyboard_event: self.keyboard_event,
            other_event: self.other_event,
            layout_event: self.layout_event,
            render_event: self.render_event,
            focus_event: self.focus_event,
            update: self.update,
        }
    }

    pub fn accept_keyboard_events<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, V::Output, B4, B5, B6, B7, B8> {
        Ignore {
            inner: self.inner,
            state_sync: self.state_sync,
            mouse_event: self.mouse_event,
            keyboard_event: value.into_read_state(),
            other_event: self.other_event,
            layout_event: self.layout_event,
            render_event: self.render_event,
            focus_event: self.focus_event,
            update: self.update,
        }
    }

    pub fn accept_other_events<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, V::Output, B5, B6, B7, B8> {
        Ignore {
            inner: self.inner,
            state_sync: self.state_sync,
            mouse_event: self.mouse_event,
            keyboard_event: self.keyboard_event,
            other_event: value.into_read_state(),
            layout_event: self.layout_event,
            render_event: self.render_event,
            focus_event: self.focus_event,
            update: self.update,
        }
    }

    pub fn accept_update<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, B4, B5, B6, B7, V::Output> {
        Ignore {
            inner: self.inner,
            state_sync: self.state_sync,
            mouse_event: self.mouse_event,
            keyboard_event: self.keyboard_event,
            other_event: self.other_event,
            layout_event: self.layout_event,
            render_event: self.render_event,
            focus_event: self.focus_event,
            update: value.into_read_state(),
        }
    }

    fn update_states(&mut self, env: &mut Environment) {
        self.state_sync.sync(env);
        self.mouse_event.sync(env);
        self.keyboard_event.sync(env);
        self.other_event.sync(env);
        self.layout_event.sync(env);
        self.render_event.sync(env);
        self.focus_event.sync(env);
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> CommonWidget for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn id(&self) -> WidgetId {
        self.inner.id()
    }

    fn flag(&self) -> WidgetFlag {
        self.inner.flag()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        self.inner.foreach_child(f)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.inner.foreach_child_mut(f)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.inner.foreach_child_rev(f)
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.inner.foreach_child_direct(f)
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.inner.foreach_child_direct_rev(f)

    }

    fn position(&self) -> Position {
        self.inner.position()
    }

    fn set_position(&mut self, position: Position) {
        self.inner.set_position(position)
    }

    fn get_focus(&self) -> Focus {
        self.inner.get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.inner.set_focus(focus)
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.inner.alignment()
    }

    fn flexibility(&self) -> u32 {
        self.inner.flexibility()
    }

    fn dimension(&self) -> Dimension {
        self.inner.dimension()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.inner.set_dimension(dimension)
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> StateSync for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.state_sync.sync(env);
        self.mouse_event.sync(env);
        self.keyboard_event.sync(env);
        self.other_event.sync(env);
        self.layout_event.sync(env);
        self.render_event.sync(env);
        self.focus_event.sync(env);
        self.update.sync(env);

        if *self.state_sync.value() {
            self.inner.capture_state(env);
        }
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.state_sync.sync(env);
        self.mouse_event.sync(env);
        self.keyboard_event.sync(env);
        self.other_event.sync(env);
        self.layout_event.sync(env);
        self.render_event.sync(env);
        self.focus_event.sync(env);
        self.update.sync(env);

        if *self.state_sync.value() {
            self.inner.release_state(env)
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> MouseEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.update_states(ctx.env);
        if *self.mouse_event.value() {
            self.inner.handle_mouse_event(event, ctx)
        }
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.update_states(ctx.env);
        if *self.mouse_event.value() {
            self.inner.process_mouse_event(event, ctx)
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> KeyboardEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.update_states(ctx.env);
        if *self.keyboard_event.value() {
            self.inner.handle_keyboard_event(event, ctx)
        }
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.update_states(ctx.env);
        if *self.keyboard_event.value() {
            self.inner.process_keyboard_event(event, ctx)
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> WindowEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.update_states(ctx.env);
        self.inner.handle_window_event(event, ctx)
    }

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.update_states(ctx.env);
        self.inner.process_window_event(event, ctx)
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> OtherEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn handle_other_event(&mut self, _event: &Event, ctx: &mut OtherEventContext) {
        self.update_states(ctx.env);
        if *self.other_event.value() {
            self.inner.handle_other_event(_event, ctx)
        }
    }

    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        self.update_states(ctx.env);
        if *self.other_event.value() {
            self.inner.process_other_event(event, ctx)
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> Update for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.update_states(ctx.env);
        if *self.update.value() {
            self.inner.update(ctx)
        }
    }

    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.update_states(ctx.env);
        if *self.update.value() {
            self.inner.process_update(ctx)
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> Layout for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.update_states(ctx.env);
        if *self.layout_event.value() {
            self.inner.calculate_size(requested_size, ctx)
        } else {
            self.inner.dimension()
        }
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.update_states(ctx.env);
        if *self.layout_event.value() {
            self.inner.position_children(ctx)
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> Render for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.update_states(env);
        if *self.render_event.value() {
            self.inner.render(context, env)
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> Focusable for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {
    fn focus_retrieved(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.update_states(env);
        if *self.focus_event.value() {
            self.inner.focus_retrieved(focus_request, env)
        }
    }

    fn focus_dismissed(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.update_states(env);
        if *self.focus_event.value() {
            self.inner.focus_dismissed(focus_request, env)
        }
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        self.update_states(env);
        if *self.focus_event.value() {
            self.inner.set_focus_and_request(focus, env)
        }
    }

    fn process_focus_request(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.update_states(env);
        if *self.focus_event.value() {
            self.inner
                .process_focus_request(focus_request, env)
        } else {
            false
        }
    }

    fn process_focus_next(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.update_states(env);
        if *self.focus_event.value() {
            self.inner
                .process_focus_next(focus_request, focus_up_for_grab, env)
        } else {
            focus_up_for_grab
        }
    }

    fn process_focus_previous(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.update_states(env);
        if *self.focus_event.value() {
            self.inner
                .process_focus_previous(focus_request, focus_up_for_grab, env)
        } else {
            focus_up_for_grab
        }
    }
}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> AnyWidget for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
> WidgetExt for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8> {}