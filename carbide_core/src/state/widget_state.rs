use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::prelude::Environment;
use crate::state::{MappedState, StateContract};
use crate::state::global_state::GlobalStateContainer;
pub use crate::state::State;
use crate::widget::GlobalStateContract;

pub struct WidgetState<T, GS>(Box<dyn State<T, GS>>);

impl<T: StateContract, GS: GlobalStateContract> WidgetState<T, GS> {
    pub fn new(item: Box<dyn State<T, GS>>) -> WidgetState<T, GS> {
        WidgetState(item)
    }
}

impl<T: StateContract + 'static, GS: GlobalStateContract> WidgetState<T, GS> {
    pub fn mapped<U: StateContract + Default + 'static>(self, map: fn(&Self) -> U) -> WidgetState<U, GS> {
        WidgetState::new(MappedState::new(self, map))
    }
}

impl<T: StateContract, GS: GlobalStateContract> Debug for WidgetState<T, GS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: StateContract, GS: GlobalStateContract> Clone for WidgetState<T, GS> {
    fn clone(&self) -> Self {
        WidgetState(self.0.clone())
    }
}

impl<T: StateContract, GS: GlobalStateContract> Into<WidgetState<T, GS>> for Box<dyn State<T, GS>> {
    fn into(self) -> WidgetState<T, GS> {
        WidgetState(self)
    }
}

impl<T: StateContract, GS: GlobalStateContract> Deref for WidgetState<T, GS> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: StateContract, GS: GlobalStateContract> DerefMut for WidgetState<T, GS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl<T: StateContract, GS: GlobalStateContract> State<T, GS> for WidgetState<T, GS> {
    fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.0.capture_state(env, global_state)
    }

    fn release_state(&mut self, env: &mut Environment<GS>) {
        self.0.release_state(env)
    }
}