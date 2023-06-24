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
use crate::layout::Layout;
use crate::render::{Primitive, Render};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent, OtherEvent)]
pub struct OverlaidLayer<C> where C: Widget + Clone {
    id: WidgetId,
    child: C,
    overlay_id: &'static str,
    overlays: Rc<RefCell<Vec<Box<dyn Widget>>>>,
    position: Position,
    dimension: Dimension,
    steal_events_when_some: bool,
}

impl OverlaidLayer<Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget + Clone>(overlay_id: &'static str, child: C) -> OverlaidLayer<C> {
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

impl<C: Widget + Clone> OverlaidLayer<C> {
    pub fn steal_events(mut self) -> OverlaidLayer<C> {
        self.steal_events_when_some = true;
        self
    }
}

impl<C: Widget + Clone> MouseEventHandler for OverlaidLayer<C> {
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

impl<C: Widget + Clone> KeyboardEventHandler for OverlaidLayer<C> {
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

impl<C: Widget + Clone> OtherEventHandler for OverlaidLayer<C> {
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

impl<C: Widget + Clone> Layout for OverlaidLayer<C> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        env.with_overlay_layer(self.overlay_id, self.overlays.clone(), |new_env| {
            self.dimension = self.child.calculate_size(requested_size, new_env);
            self.dimension
        })
    }
}

impl<C: Widget + Clone> CommonWidget for OverlaidLayer<C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget + Clone> Render for OverlaidLayer<C> {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        /*self.foreach_child_mut(&mut |child| {
            child.process_get_primitives(primitives, env);
        });

        // If we have an overlay in the env
        if let Some(overlay) = env.overlay(&self.overlay_id) {
            // If we already contained an overlay, set its showing to false
            if let Some(overlay) = &mut self.overlay {
                overlay.set_showing(false);
            }
            // Insert the overlay
            self.overlay = overlay;
            // If there is a new overlay put in, set its showing to true
            if let Some(overlay) = &mut self.overlay {
                overlay.set_showing(true);
            }
        }

        if let Some(t) = &mut self.overlay {
            t.process_get_primitives(primitives, env);
        }*/
    }

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

impl<C: Widget + Clone> WidgetExt for OverlaidLayer<C> {}
