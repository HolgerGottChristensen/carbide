use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::environment::environment::Environment;
use crate::state::{State, StateContract, TState};
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::state::widget_state::WidgetState;

/// Warning. This state can not be used to modify the environment, as it pulls the value out
/// of the environment at the time the state is captured. If the value is modified, the
/// modification is lost the next time the state is captured.
#[derive(Clone)]
pub struct EnvState<T, GS> where T: StateContract, GS: GlobalStateContract {
    map: fn(env: &Environment<GS>) -> T,
    value: T,
}

impl<T: StateContract + Default, GS: GlobalStateContract> EnvState<T, GS> {
    pub fn new(map: fn(env: &Environment<GS>) -> T) -> Self {
        EnvState {
            map,
            value: T::default(),
        }
    }
}

impl<T: StateContract, GS: GlobalStateContract> Deref for EnvState<T, GS> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: StateContract, GS: GlobalStateContract> DerefMut for EnvState<T, GS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: StateContract, GS: GlobalStateContract> State<T, GS> for EnvState<T, GS> {
    fn capture_state(&mut self, env: &mut Environment<GS>, _: &GlobalStateContainer<GS>) {
        self.value = (self.map)(env);
    }

    fn release_state(&mut self, _: &mut Environment<GS>) {}
}

impl<T: StateContract, GS: GlobalStateContract> Debug for EnvState<T, GS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::EnvState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract + 'static, GS: GlobalStateContract> Into<TState<T, GS>> for Box<EnvState<T, GS>> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(self)
    }
}