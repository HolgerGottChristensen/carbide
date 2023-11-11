use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{
    KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler,
    WidgetEvent,
};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Primitive, Render};
use crate::widget::{CommonWidget, Empty, AnyWidget, WidgetExt, WidgetId, Widget};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent, OtherEvent)]
pub struct OverlaidLayer<C> where C: Widget {
    id: WidgetId,
    child: C,
    overlay_id: &'static str,
    overlays: Rc<RefCell<Vec<Box<dyn AnyWidget>>>>,
    position: Position,
    dimension: Dimension,
    steal_events_when_some: bool,
}

impl OverlaidLayer<Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget>(overlay_id: &'static str, child: C) -> OverlaidLayer<C> {
        OverlaidLayer {
            id: WidgetId::new(),
            child,
            overlay_id,
            overlays: Rc::new(RefCell::new(vec![])),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            steal_events_when_some: false,
        }
    }
}

impl<C: Widget> OverlaidLayer<C> {
    pub fn steal_events(mut self) -> OverlaidLayer<C> {
        self.steal_events_when_some = true;
        self
    }
}

impl<C: Widget> MouseEventHandler for OverlaidLayer<C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        let mut widgets = self.overlays.borrow_mut();
        for widget in widgets.deref_mut() {
            widget.process_mouse_event(event, consumed, env)
        }

        if self.steal_events_when_some && widgets.len() > 0 {
            return;
        }

        drop(widgets);

        env.with_overlay_layer(self.overlay_id, self.overlays.clone(), |new_env| {
            self.child.process_mouse_event(event, consumed, new_env)
        });
    }
}

impl<C: Widget> KeyboardEventHandler for OverlaidLayer<C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        let mut widgets = self.overlays.borrow_mut();
        for widget in widgets.deref_mut() {
            widget.process_keyboard_event(event, env)
        }

        if self.steal_events_when_some && widgets.len() > 0 {
            return;
        }

        drop(widgets);

        env.with_overlay_layer(self.overlay_id, self.overlays.clone(), |new_env| {
            self.child.process_keyboard_event(event, new_env)
        });
    }
}

impl<C: Widget> OtherEventHandler for OverlaidLayer<C> {
    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        let mut widgets = self.overlays.borrow_mut();
        for widget in widgets.deref_mut() {
            widget.process_other_event(event, env)
        }

        if self.steal_events_when_some && widgets.len() > 0 {
            return;
        }

        drop(widgets);

        env.with_overlay_layer(self.overlay_id, self.overlays.clone(), |new_env| {
            self.child.process_other_event(event, new_env)
        });
    }
}

impl<C: AnyWidget + Clone> Layout for OverlaidLayer<C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.dimension = self.child.calculate_size(requested_size, ctx);
        self.dimension

        // ctx.env.with_overlay_layer(self.overlay_id, self.overlays.clone(), |new_env| {
        //     self.dimension = self.child.calculate_size(requested_size, new_env);
        //     self.dimension
        // })
    }
}

impl<C: AnyWidget + Clone> CommonWidget for OverlaidLayer<C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: AnyWidget + Clone> Render for OverlaidLayer<C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {

        /*env.with_overlay_layer(self.overlay_id, self.overlays.clone(), |new_env| {
            self.child.render(context, new_env)
        });*/

        self.child.render(context, env);

        let mut widgets = self.overlays.borrow_mut();
        for widget in widgets.deref_mut() {
            widget.render(context, env)
        }
    }
}

impl<C: AnyWidget + Clone> WidgetExt for OverlaidLayer<C> {}
