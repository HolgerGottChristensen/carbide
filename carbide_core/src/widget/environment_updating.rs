use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::{Environment, EnvironmentStateContainer, EnvironmentVariable};
use crate::event::{KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WidgetEvent};
use crate::focus::{Focusable, Refocus};
use crate::render::Render;
use crate::state::{NewStateSync, ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_derive(Layout, StateSync)]
pub struct EnvUpdating<C> where C: Widget {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    envs_to_update: Vec<EnvironmentStateContainer>,
}

impl EnvUpdating<Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget>(child: C) -> EnvUpdating<C> {
        EnvUpdating {
            id: WidgetId::new(),
            child,
            position: Position::default(),
            dimension: Dimension::default(),
            envs_to_update: vec![],
        }
    }
}

impl<C: Widget> EnvUpdating<C> {
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
                    env.push(EnvironmentVariable::EnvironmentColor {
                        key: key.clone(),
                        value: to_update,
                    });
                    //value.release_state(env);
                }
                EnvironmentStateContainer::FontSize { key, value } => {
                    value.sync(env);
                    let to_update = *value.value();

                    env.push(EnvironmentVariable::EnvironmentFontSize {
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
                EnvironmentStateContainer::Bool { key, value } => {
                    value.sync(env);
                    env.push(EnvironmentVariable::Bool {
                        key,
                        value: *value.value(),
                    });
                }
            }
        }
    }
}

impl<C: Widget> OtherEventHandler for EnvUpdating<C> {
    fn process_other_event(&mut self, event: &WidgetEvent, ctx: &mut OtherEventContext) {
        self.insert_into_env(ctx.env);

        self.child.process_other_event(event, ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget> KeyboardEventHandler for EnvUpdating<C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.insert_into_env(env);

        self.child.process_keyboard_event(event, env);

        self.remove_from_env(env);
    }
}

impl<C: Widget> MouseEventHandler for EnvUpdating<C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, ctx: &mut MouseEventContext) {
        self.insert_into_env(ctx.env);

        self.child.process_mouse_event(event, &consumed, ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget> Focusable for EnvUpdating<C> {
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

impl<C: Widget> Render for EnvUpdating<C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.insert_into_env(env);

        self.foreach_child_mut(&mut |child| {
            child.render(context, env);
        });

        self.remove_from_env(env);
    }
}

impl<C: Widget> CommonWidget for EnvUpdating<C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<C: Widget> WidgetExt for EnvUpdating<C> {}
