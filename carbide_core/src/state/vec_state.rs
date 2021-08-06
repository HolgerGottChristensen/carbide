use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::prelude::Environment;
use crate::prelude::GlobalStateContract;
use crate::state::{StateContract, TState, UsizeState};
use crate::state::global_state::GlobalStateContainer;
use crate::state::state::State;
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct VecState<T, GS> where T: StateContract, GS: GlobalStateContract {
    index_state: UsizeState<GS>,
    vec: TState<Vec<T>, GS>,
}

impl<T: StateContract, GS: GlobalStateContract> VecState<T, GS> {
    pub fn new<I: Into<UsizeState<GS>>, V: Into<TState<Vec<T>, GS>>>(index: I, vec: V) -> VecState<T, GS> {
        VecState {
            index_state: index.into(),
            vec: vec.into(),
        }
    }
}

impl<T: StateContract, GS: GlobalStateContract> Deref for VecState<T, GS> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.vec[*self.index_state]
    }
}

impl<T: StateContract, GS: GlobalStateContract> DerefMut for VecState<T, GS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec[*self.index_state]
    }
}

impl<T: StateContract, GS: GlobalStateContract> State<T, GS> for VecState<T, GS> {
    fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.index_state.capture_state(env, global_state);
        self.vec.capture_state(env, global_state);
    }

    fn release_state(&mut self, env: &mut Environment<GS>) {
        self.index_state.release_state(env);
        self.vec.release_state(env);
    }
}

impl<T: StateContract, GS: GlobalStateContract> Debug for VecState<T, GS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::VecState")
            .field("value", self.deref())
            .field("index", self.index_state.deref())
            .finish()
    }
}

impl<T: StateContract + 'static, GS: GlobalStateContract> Into<TState<T, GS>> for VecState<T, GS> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract + 'static, GS: GlobalStateContract> Into<TState<T, GS>> for Box<VecState<T, GS>> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(self)
    }
}