use carbide::environment::EnvironmentStack;
use carbide::focus::{Focus, FocusManager, Refocus};
use carbide::state::{ReadState, State};
use carbide::widget::{MouseAreaAction, MouseAreaActionContext};

#[derive(Debug, Clone)]
pub struct PickerAction<C, F, E> where
    C: State<T=bool>,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
{
    pub value: C,
    pub focus: F,
    pub enabled: E,
}

impl<C: State<T=bool>, F: State<T=Focus>, E: ReadState<T=bool>> PickerAction<C, F, E> {
    fn trigger(&mut self, env_stack: &mut EnvironmentStack) {
        self.enabled.sync(env_stack);

        if !*self.enabled.value() {
            return;
        }

        self.focus.sync(env_stack);
        self.value.sync(env_stack);

        let val = !*self.value.value();
        *self.value.value_mut() = val;

        if *self.focus.value() != Focus::Focused {
            *self.focus.value_mut() = Focus::FocusRequested;
            FocusManager::get(env_stack, |manager| {
                manager.request_focus(Refocus::FocusRequest)
            });
        }
    }
}

impl<C: State<T=bool>, F: State<T=Focus>, E: ReadState<T=bool>> MouseAreaAction for PickerAction<C, F, E> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.trigger(ctx.env_stack)
    }
}