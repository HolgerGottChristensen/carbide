use crate::draw::{Dimension, Position};
use crate::event::{KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::prelude::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent, OtherEvent)]
pub struct OverlaidLayer {
    id: Uuid,
    child: Box<dyn Widget>,
    overlay: Option<Overlay>,
    overlay_id: String,
    position: Position,
    dimension: Dimension,
    steal_events_when_some: bool,
}

impl OverlaidLayer {
    pub fn new(overlay_id: &str, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
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
        if !*consumed {
            self.capture_state(env);
            self.handle_mouse_event(event, consumed, env);
            self.release_state(env);
        }

        if let Some(overlay) = &mut self.overlay {
            overlay.process_mouse_event(event, consumed, env);
            if *consumed { return (); }

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
        self.capture_state(env);
        self.handle_keyboard_event(event, env);
        self.release_state(env);

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
        self.capture_state(env);
        self.handle_other_event(event, env);
        self.release_state(env);

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

impl CommonWidget for OverlaidLayer {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        0
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

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

#[derive(Debug, Clone)]
pub enum OverlayValue {
    Insert(Box<Overlay>),
    Update(Position, Dimension),
    Remove,
}
