use crate::toggle::toggle_value::ToggleValue;
use carbide::environment::EnvironmentStack;
use carbide::focus::{Focus, FocusManager, Refocus};
use carbide::state::{ReadState, State};
use carbide::widget::{MouseAreaAction, MouseAreaActionContext};

#[derive(Debug, Clone)]
pub struct ToggleAction<C, F, E> where
    C: State<T=ToggleValue>,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
{
    pub value: C,
    pub focus: F,
    pub enabled: E,
}

impl<C: State<T=ToggleValue>, F: State<T=Focus>, E: ReadState<T=bool>> ToggleAction<C, F, E> {
    pub(crate) fn trigger(&mut self, env_stack: &mut EnvironmentStack) {
        self.enabled.sync(env_stack);

        if !*self.enabled.value() {
            return;
        }

        self.focus.sync(env_stack);
        self.value.sync(env_stack);

        if *self.value.value() == ToggleValue::True {
            *self.value.value_mut() = ToggleValue::False;
        } else {
            *self.value.value_mut() = ToggleValue::True;
        }

        if *self.focus.value() != Focus::Focused {
            *self.focus.value_mut() = Focus::FocusRequested;
            FocusManager::get(env_stack, |manager| {
                manager.request_focus(Refocus::FocusRequest)
            });
        }
    }
}

impl<C: State<T=ToggleValue>, F: State<T=Focus>, E: ReadState<T=bool>> MouseAreaAction for ToggleAction<C, F, E> {
    fn call(&mut self, ctx: MouseAreaActionContext) { self.trigger(ctx.env_stack) }
}