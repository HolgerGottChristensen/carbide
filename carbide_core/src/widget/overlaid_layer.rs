use carbide_macro::carbide_default_builder;
use crate::draw::{Dimension, Position};
use crate::event::{
    KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler,
    WidgetEvent,
};
use crate::CommonWidgetImpl;
use crate::environment::Environment;
use crate::layout::Layout;
use crate::render::{Primitive, Render};
use crate::widget::{CommonWidget, Overlay, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent, OtherEvent)]
pub struct OverlaidLayer {
    id: WidgetId,
    child: Box<dyn Widget>,
    overlay: Option<Overlay>,
    overlay_id: String,
    position: Position,
    dimension: Dimension,
    steal_events_when_some: bool,
}

impl OverlaidLayer {
    #[carbide_default_builder]
    pub fn new(overlay_id: &str, child: Box<dyn Widget>) -> Box<Self> {}

    pub fn new(overlay_id: &str, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Self {
            id: WidgetId::new(),
            child,
            overlay: None,
            overlay_id: overlay_id.to_string(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            steal_events_when_some: false,
        })
    }

    pub fn steal_events(mut self) -> Box<Self> {
        self.steal_events_when_some = true;
        Box::new(self)
    }
}

impl MouseEventHandler for OverlaidLayer {
    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {

        if let Some(overlay) = &mut self.overlay {
            overlay.process_mouse_event(event, consumed, env);
            if *consumed {
                return ();
            }

            if !self.steal_events_when_some {
                for mut child in self.children_direct() {
                    child.process_mouse_event(event, &consumed, env);
                    if *consumed {
                        return ();
                    }
                }
            }
        } else {
            for mut child in self.children_direct() {
                child.process_mouse_event(event, &consumed, env);
                if *consumed {
                    return ();
                }
            }
        }
    }
}

impl KeyboardEventHandler for OverlaidLayer {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {

        if let Some(overlay) = &mut self.overlay {
            overlay.process_keyboard_event(event, env);
            if !self.steal_events_when_some {
                for mut child in self.children_direct() {
                    child.process_keyboard_event(event, env);
                }
            }
        } else {
            for mut child in self.children_direct() {
                child.process_keyboard_event(event, env);
            }
        }
    }
}

impl OtherEventHandler for OverlaidLayer {
    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {

        if let Some(overlay) = &mut self.overlay {
            overlay.process_other_event(event, env);
            if !self.steal_events_when_some {
                for mut child in self.children_direct() {
                    child.process_other_event(event, env);
                }
            }
        } else {
            for mut child in self.children_direct() {
                child.process_other_event(event, env);
            }
        }
    }
}

impl Layout for OverlaidLayer {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.dimension = self.child.calculate_size(requested_size, env);
        self.dimension
    }
}

CommonWidgetImpl!(OverlaidLayer, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);

impl Render for OverlaidLayer {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

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
        }
    }
}

impl WidgetExt for OverlaidLayer {}
