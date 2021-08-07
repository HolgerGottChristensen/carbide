use crate::event::event::Event;
use crate::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::prelude::*;
use crate::state::global_state::GlobalStateContainer;
use crate::widget::render::RenderProcessor;

#[derive(Debug, Clone, Widget)]
//#[state_sync(sync_state, update_all_widget_state, update_local_widget_state)]
//#[event(process_keyboard_event, process_mouse_event, process_other_event)]
#[render(process_get_primitives)]
pub struct OverlaidLayer {
    id: Uuid,
    child: Box<dyn Widget>,
    overlay: Option<Box<dyn Widget>>,
    current_overlay_id: Option<Uuid>,
    overlay_id: String,
    position: Point,
    dimension: Dimensions,
    steal_events_when_some: bool,
}

impl OverlaidLayer {
    /*fn update_all_widget_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
        if let Some(overlay) = &mut self.overlay {
            overlay.update_all_widget_state(env, global_state);
        }
    }

    fn update_local_widget_state(&mut self, env: &Environment<GS>) {
        if let Some(overlay) = &mut self.overlay {
            overlay.update_local_widget_state(env);
        }
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &mut GS) {
        self.update_all_widget_state(env, global_state);

        if !*consumed {
            self.handle_mouse_event(event, consumed, env, global_state);
        }

        self.insert_local_state(env);


        if let Some(overlay) = &mut self.overlay {
            overlay.process_mouse_event(event, &consumed, env, global_state);
            if *consumed { return (); }

            if !self.steal_events_when_some {
                for child in self.get_proxied_children() {
                    child.process_mouse_event(event, &consumed, env, global_state);
                    if *consumed { return (); }
                }
            }
        } else {
            for child in self.get_proxied_children() {
                child.process_mouse_event(event, &consumed, env, global_state);
                if *consumed { return (); }
            }
        }

        self.update_local_widget_state(env)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        self.update_all_widget_state(env, global_state);

        self.handle_keyboard_event(event, env, global_state);

        self.insert_local_state(env);

        if let Some(overlay) = &mut self.overlay {
            overlay.process_keyboard_event(event, env, global_state);

            if !self.steal_events_when_some {
                for child in self.get_proxied_children() {
                    child.process_keyboard_event(event, env, global_state);
                }
            }
        } else {
            for child in self.get_proxied_children() {
                child.process_keyboard_event(event, env, global_state);
            }
        }

        self.update_local_widget_state(env)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        self.update_all_widget_state(env, global_state);

        self.insert_local_state(env);

        if let Some(overlay) = &mut self.overlay {
            overlay.process_other_event(event, env, global_state);

            if !self.steal_events_when_some {
                for child in self.get_proxied_children() {
                    child.process_other_event(event, env, global_state);
                }
            }
        } else {
            for child in self.get_proxied_children() {
                child.process_other_event(event, env, global_state);
            }
        }

        self.update_local_widget_state(env)
    }

    fn sync_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
        // This might not be the prettiest place to retrieve things from the env
        self.update_all_widget_state(env, global_state);

        self.insert_local_state(env);

        if let Some(overlay) = &mut self.overlay {
            overlay.sync_state(env, global_state);
        }

        for child in self.get_proxied_children() {
            child.sync_state(env, global_state)
        }

        // Check if env contains an overlay widget with the specified id
        let temp_overlay = env.get_overlay(&self.overlay_id);

        if let Some(overlay) = temp_overlay {
            if let Some(current_overlay_id) = self.current_overlay_id {

                // If the overlay has the same id as the overlay shown last frame we should not
                // update it, because we will loose its state.
                if overlay.get_id() != current_overlay_id {
                    self.overlay = Some(overlay);
                    self.current_overlay_id = Some(current_overlay_id)
                } else {
                    if let Some(o) = &mut self.overlay {
                        o.set_position(overlay.get_position());
                    }
                }
            } else {
                self.current_overlay_id = Some(overlay.get_id());
                self.overlay = Some(overlay);
            }
        } else {
            self.current_overlay_id = None;
            self.overlay = None;
        }

        if let Some(overlay) = &mut self.overlay {
            overlay.sync_state(env, global_state);
        }


        self.update_local_widget_state(env);
    }*/

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        for child in self.get_children_mut() {
            child.process_get_primitives(primitives, env);
        }

        if let Some(t) = &mut self.overlay {
            t.process_get_primitives(primitives, env);
        }
    }

    pub fn new(overlay_id: &str, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child,
            overlay: None,
            current_overlay_id: None,
            overlay_id: overlay_id.to_string(),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            steal_events_when_some: true,
        })
    }
}


impl Layout for OverlaidLayer {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, env);

        if let Some(overlay) = &mut self.overlay {
            overlay.calculate_size(requested_size, env);
        }

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
        if let Some(overlay) = &mut self.overlay {
            overlay.position_children();
        }
    }
}

impl CommonWidget for OverlaidLayer {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl Render for OverlaidLayer {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        return vec![];
    }
}


impl WidgetExt for OverlaidLayer {}