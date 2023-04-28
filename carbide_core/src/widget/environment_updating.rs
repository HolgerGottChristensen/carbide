use carbide_core::render::RenderContext;

use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::{Environment, EnvironmentStateContainer, EnvironmentVariable};
use crate::event::{
    KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler,
    WidgetEvent,
};
use crate::focus::{Focusable, Refocus};
use crate::render::{Primitive, Render};
use crate::state::{NewStateSync, ReadState};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId};

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
    #[carbide_default_builder]
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {}

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

                    env.push(EnvironmentVariable::String {
                        key: key.clone(),
                        value: to_update,
                    });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::U32 { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::U32 {
                        key: key.clone(),
                        value: to_update,
                    });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::F64 { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::F64 {
                        key: key.clone(),
                        value: to_update,
                    });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::Color { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();
                    env.push(EnvironmentVariable::Color {
                        key: key.clone(),
                        value: to_update,
                    });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::FontSize { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::FontSize {
                        key: key.clone(),
                        value: to_update,
                    });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::I32 { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::I32 {
                        key: key.clone(),
                        value: to_update,
                    });
                    //value.release_state(env);
                }
            }
        }
    }
}

impl OtherEventHandler for EnvUpdating {
    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.insert_into_env(env);

        self.child.process_other_event(event, env);

        self.remove_from_env(env);
    }
}

impl KeyboardEventHandler for EnvUpdating {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.insert_into_env(env);

        self.child.process_keyboard_event(event, env);

        self.remove_from_env(env);
    }
}

impl MouseEventHandler for EnvUpdating {
    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.insert_into_env(env);

        self.child.process_mouse_event(event, &consumed, env);

        self.remove_from_env(env);
    }
}

impl Focusable for EnvUpdating {
    fn process_focus_request(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.insert_into_env(env);

        let any_focus = self.child.process_focus_request(event, focus_request, env);

        self.remove_from_env(env);

        any_focus
    }

    fn process_focus_next(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.insert_into_env(env);

        let focus_child = self.child.process_focus_next(event, focus_request, focus_up_for_grab, env);

        self.remove_from_env(env);

        focus_child
    }

    fn process_focus_previous(
        &mut self,
        event: &WidgetEvent,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.insert_into_env(env);

        let focus_child = self.child.process_focus_previous(event, focus_request, focus_up_for_grab, env);

        self.remove_from_env(env);

        focus_child
    }
}

impl Render for EnvUpdating {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.insert_into_env(env);

        self.foreach_child_mut(&mut |child| {
            child.render(context, env);
        });

        self.remove_from_env(env);
    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.insert_into_env(env);

        self.foreach_child_mut(&mut |child| {
            child.process_get_primitives(primitives, env);
        });

        self.remove_from_env(env);
    }
}

impl CommonWidget for EnvUpdating {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl WidgetExt for EnvUpdating {}
