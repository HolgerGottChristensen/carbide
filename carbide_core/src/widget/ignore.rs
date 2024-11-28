use std::fmt::Debug;
use carbide::accessibility::AccessibilityContext;
use carbide::environment::EnvironmentStack;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext};
use carbide::lifecycle::Initialize;
use carbide::widget::Identifiable;
use crate::accessibility::Accessibility;
use crate::draw::{Alignment, Dimension, Position};
use crate::environment::Environment;
use crate::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler, Event, AccessibilityEventHandler};
use crate::flags::WidgetFlag;
use crate::focus::{Focus, Focusable, FocusContext};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, StateSync, ReadState};
use crate::lifecycle::{Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetSync};

#[derive(Clone, Debug)]
pub struct Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> where
    T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
    B9: ReadState<T=bool>,
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
    accessibility_event: B9,
    accessibility: B10,
}

impl Ignore<Empty, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool> {
    pub fn new<T: Widget>(widget: T) -> Ignore<T, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool> {
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
            accessibility_event: true,
            accessibility: true
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    pub fn render<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, B4, B5, V::Output, B7, B8, B9, B10> {
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
            accessibility_event: self.accessibility_event,
            accessibility: self.accessibility,
        }
    }

    pub fn layout<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, B4, V::Output, B6, B7, B8, B9, B10> {
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
            accessibility_event: self.accessibility_event,
            accessibility: self.accessibility,
        }
    }

    pub fn accept_mouse_events<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, V::Output, B3, B4, B5, B6, B7, B8, B9, B10> {
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
            accessibility_event: self.accessibility_event,
            accessibility: self.accessibility,
        }
    }

    pub fn accept_keyboard_events<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, V::Output, B4, B5, B6, B7, B8, B9, B10> {
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
            accessibility_event: self.accessibility_event,
            accessibility: self.accessibility,
        }
    }

    pub fn accept_other_events<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, V::Output, B5, B6, B7, B8, B9, B10> {
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
            accessibility_event: self.accessibility_event,
            accessibility: self.accessibility,
        }
    }

    pub fn accept_update<V: IntoReadState<bool>>(self, value: V) -> Ignore<T, B1, B2, B3, B4, B5, B6, B7, V::Output, B9, B10> {
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
            accessibility_event: self.accessibility_event,
            accessibility: self.accessibility,
        }
    }

    fn update_states(&mut self, env: &mut EnvironmentStack) {
        self.state_sync.sync(env);
        self.mouse_event.sync(env);
        self.keyboard_event.sync(env);
        self.other_event.sync(env);
        self.layout_event.sync(env);
        self.render_event.sync(env);
        self.focus_event.sync(env);
        self.update.sync(env);
        self.accessibility_event.sync(env);
        self.accessibility.sync(env);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Identifiable for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn id(&self) -> WidgetId {
        self.inner.id()
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> CommonWidget for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
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

    fn alignment(&self) -> Alignment {
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> WidgetSync for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn sync(&mut self, env: &mut EnvironmentStack) {
        self.state_sync.sync(env);
        self.mouse_event.sync(env);
        self.keyboard_event.sync(env);
        self.other_event.sync(env);
        self.layout_event.sync(env);
        self.render_event.sync(env);
        self.focus_event.sync(env);
        self.update.sync(env);

        if *self.state_sync.value() {
            self.inner.sync(env);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> MouseEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.update_states(ctx.env_stack);
        if *self.mouse_event.value() {
            self.inner.handle_mouse_event(event, ctx)
        }
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.update_states(ctx.env_stack);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> KeyboardEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.update_states(ctx.env_stack);
        if *self.keyboard_event.value() {
            self.inner.handle_keyboard_event(event, ctx)
        }
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.update_states(ctx.env_stack);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> WindowEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.update_states(ctx.env_stack);
        self.inner.handle_window_event(event, ctx)
    }

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.update_states(ctx.env_stack);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> OtherEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn handle_other_event(&mut self, _event: &Event, ctx: &mut OtherEventContext) {
        self.update_states(ctx.env_stack);
        if *self.other_event.value() {
            self.inner.handle_other_event(_event, ctx)
        }
    }

    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        self.update_states(ctx.env_stack);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> AccessibilityEventHandler for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.update_states(ctx.env_stack);
        if *self.mouse_event.value() {
            self.inner.handle_accessibility_event(event, ctx)
        }
    }

    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.update_states(ctx.env_stack);
        if *self.mouse_event.value() {
            self.inner.process_accessibility_event(event, ctx)
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Update for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.update_states(ctx.env_stack);
        if *self.update.value() {
            self.inner.update(ctx)
        }
    }

    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.update_states(ctx.env_stack);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Layout for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.update_states(ctx.env_stack);
        if *self.layout_event.value() {
            self.inner.calculate_size(requested_size, ctx)
        } else {
            self.inner.dimension()
        }
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.update_states(ctx.env_stack);
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Render for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn render(&mut self, context: &mut RenderContext) {
        self.update_states(context.env_stack);
        if *self.render_event.value() {
            self.inner.render(context)
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Focusable for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn request_focus(&mut self, env: &mut EnvironmentStack) {
        //self.update_states(env);
        if *self.focus_event.value() {
            self.inner.request_focus(env)
        }
    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        self.update_states(ctx.env_stack);
        if *self.focus_event.value() {
            self.inner.process_focus_request(ctx)
        }
    }

    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        self.update_states(ctx.env_stack);
        if *self.focus_event.value() {
            self.inner.process_focus_next(ctx)
        }
    }

    fn process_focus_previous(
        &mut self,
        ctx: &mut FocusContext,
    ) {
        self.update_states(ctx.env_stack);
        if *self.focus_event.value() {
            self.inner.process_focus_previous(ctx)
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Accessibility for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.update_states(ctx.env_stack);
        if *self.accessibility.value() {
            self.inner.process_accessibility(ctx)
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
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> Initialize for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {}

impl<T: Widget,
    B1: ReadState<T=bool>,
    B2: ReadState<T=bool>,
    B3: ReadState<T=bool>,
    B4: ReadState<T=bool>,
    B5: ReadState<T=bool>,
    B6: ReadState<T=bool>,
    B7: ReadState<T=bool>,
    B8: ReadState<T=bool>,
    B9: ReadState<T=bool>,
    B10: ReadState<T=bool>,
> AnyWidget for Ignore<T, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}