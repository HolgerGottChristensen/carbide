use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::event::{KeyboardEventHandler, MouseEventHandler, OtherEventHandler};
use crate::flags::Flags;
use crate::focus::{Focus, Focusable};
use crate::focus::Refocus;
use crate::layout::{Layout, LayoutContext, Layouter};
use crate::render::{Primitive, RenderContext};
use crate::render::Render;
use crate::state::{IntoReadState, ReadState, StateSync};
use crate::widget::{CommonWidget, Duplicated, Empty, Ignore, AnyWidget, WidgetExt, WidgetId, Widget};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone)]
pub struct Overlay<W, B> where W: Widget, B: ReadState<T=bool> {
    overlay: Ignore<Duplicated<W>, bool, bool, bool, bool, bool, bool, bool>,
    hierarchy: Ignore<Duplicated<W>, bool, bool, bool, bool, bool, bool, bool>,
    showing: B,
    layer_id: &'static str,
}

impl Overlay<Empty, bool> {
    pub fn new<W: AnyWidget + Clone, B: IntoReadState<bool>>(layer: &'static str, showing: B, child: W) -> Overlay<W, B::Output> {
        let dup = Duplicated::new(child);

        let hierarchy = Ignore::new(dup.duplicate())
            .render(false)
            .accept_keyboard_events(false)
            .accept_mouse_events(false)
            .accept_other_events(false);

        let overlay = Ignore::new(dup.duplicate()).layout(false);


        Overlay {
            overlay,
            hierarchy,
            showing: showing.into_read_state(),
            layer_id: layer,
        }
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>>  Overlay<W, B> {
    fn ensure_overlay_correct(&mut self, env: &mut Environment) {
        self.showing.sync(env);

        // TODO: Optimize to not add and remove every time this is called
        if *self.showing.value() {
            env.add_overlay(self.layer_id, Box::new(self.overlay.clone()));
        } else {
            env.remove_overlay(self.layer_id, self.overlay.id());
        }
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> CommonWidget for Overlay<W, B> {
    fn id(&self) -> WidgetId {
        self.hierarchy.id()
    }

    fn flag(&self) -> Flags {
        self.hierarchy.flag()
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.hierarchy.alignment()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        self.hierarchy.foreach_child(f)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.hierarchy.foreach_child_mut(f)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.hierarchy.foreach_child_rev(f)
    }

    fn position(&self) -> Position {
        self.hierarchy.position()
    }

    fn set_position(&mut self, position: Position) {
        self.hierarchy.set_position(position)
    }

    fn get_focus(&self) -> Focus {
        self.hierarchy.get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.hierarchy.set_focus(focus)
    }

    fn flexibility(&self) -> u32 {
        self.hierarchy.flexibility()
    }

    fn dimension(&self) -> Dimension {
        self.hierarchy.dimension()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.hierarchy.set_dimension(dimension)
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.hierarchy.foreach_child_direct(f)
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.hierarchy.foreach_child_direct_rev(f)
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> MouseEventHandler for Overlay<W, B> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.handle_mouse_event(event, consumed, env)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.process_mouse_event(event, consumed, env)
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> KeyboardEventHandler for Overlay<W, B> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.handle_keyboard_event(event, env)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.process_keyboard_event(event, env)
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> OtherEventHandler for Overlay<W, B> {
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.handle_other_event(event, env)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.process_other_event(event, env)
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> StateSync for Overlay<W, B> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.ensure_overlay_correct(env);
        self.hierarchy.release_state(env)
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> Layout for Overlay<W, B> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        todo!()
        // self.ensure_overlay_correct(ctx);
        // self.hierarchy.calculate_size(requested_size, ctx)
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.hierarchy.position_children(ctx)
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> Render for Overlay<W, B> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        Render::render(&mut self.hierarchy, context, env)
    }
}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> Focusable for Overlay<W, B> {
    fn focus_retrieved(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.hierarchy.focus_retrieved(event, focus_request, env)
    }

    fn focus_dismissed(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
        self.hierarchy.focus_dismissed(event, focus_request, env)
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        self.hierarchy.set_focus_and_request(focus, env)
    }

    fn process_focus_request(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.ensure_overlay_correct(env);
        self.hierarchy
            .process_focus_request(event, focus_request, env)
    }

    fn process_focus_next(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.ensure_overlay_correct(env);
        self.hierarchy
            .process_focus_next(event, focus_request, focus_up_for_grab, env)
    }

    fn process_focus_previous(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.ensure_overlay_correct(env);
        self.hierarchy
            .process_focus_previous(event, focus_request, focus_up_for_grab, env)
    }
}


impl<W: AnyWidget + Clone, B: ReadState<T=bool>> AnyWidget for Overlay<W, B> {}

impl<W: AnyWidget + Clone, B: ReadState<T=bool>> WidgetExt for Overlay<W, B> {}