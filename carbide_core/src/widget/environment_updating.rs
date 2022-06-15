use crate::draw::{Dimension, Position};
use crate::event::{KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::focus::{Focusable, Refocus};
use crate::prelude::*;
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
#[carbide_derive(Layout, StateSync)]
pub struct EnvUpdating {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    envs_to_update: Vec<EnvironmentStateContainer>,
}

impl EnvUpdating {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(EnvUpdating {
            id: WidgetId::new(),
            child,
            position: Position::default(),
            dimension: Dimension::default(),
            envs_to_update: vec![],
        })
    }

    pub fn add(&mut self, env_to_update: EnvironmentStateContainer) {
        self.envs_to_update.push(env_to_update);
    }

    fn remove_from_env(&self, env: &mut Environment) {
        for _ in &self.envs_to_update {
            env.pop()
        }
    }

    fn insert_into_env(&mut self, env: &mut Environment) {
        for env_to_update in &mut self.envs_to_update {
            match env_to_update {
                EnvironmentStateContainer::String { key, value } => {
                    value.sync(env);
                    let to_update = value.value().clone();

                    env.push(EnvironmentVariable::String { key: key.clone(), value: to_update });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::U32 { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::U32 { key: key.clone(), value: to_update });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::F64 { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::F64 { key: key.clone(), value: to_update });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::Color { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();
                    env.push(EnvironmentVariable::Color { key: key.clone(), value: to_update });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::FontSize { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::FontSize { key: key.clone(), value: to_update });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::I32 { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::I32 { key: key.clone(), value: to_update });
                    //value.release_state(env);
                }
            }
        }
    }
}

impl OtherEventHandler for EnvUpdating {
    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.insert_into_env(env);

        for mut child in self.children_direct() {
            child.process_other_event(event, env);
        }

        self.remove_from_env(env);
    }
}

impl KeyboardEventHandler for EnvUpdating {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.insert_into_env(env);

        for mut child in self.children_direct() {
            child.process_keyboard_event(event, env);
        }

        self.remove_from_env(env);
    }
}

impl MouseEventHandler for EnvUpdating {
    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.insert_into_env(env);
        for mut child in self.children_direct() {
            child.process_mouse_event(event, &consumed, env);
            if *consumed {
                break
            }
        }

        self.remove_from_env(env);
    }
}

impl Focusable for EnvUpdating {
    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment) -> bool {
        let mut any_focus = false;
        self.insert_into_env(env);

        for mut child in self.children_direct() {
            if child.process_focus_request(event, focus_request, env) {
                any_focus = true;
            }
        }

        self.remove_from_env(env);
        any_focus
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        let mut focus_child = focus_up_for_grab;
        self.insert_into_env(env);
        for mut child in self.children_direct() {
            focus_child = child.process_focus_next(event, focus_request, focus_child, env);
        }
        self.remove_from_env(env);
        focus_child
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        let mut focus_child = focus_up_for_grab;
        self.insert_into_env(env);
        for mut child in self.children_direct_rev() {
            focus_child = child.process_focus_previous(event, focus_request, focus_child, env);
        }
        self.remove_from_env(env);
        focus_child
    }
}

impl Render for EnvUpdating {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.insert_into_env(env);

        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        self.remove_from_env(env);
    }
}

CommonWidgetImpl!(EnvUpdating, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for EnvUpdating {}
