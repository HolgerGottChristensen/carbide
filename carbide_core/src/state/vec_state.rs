use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::prelude::Environment;
use crate::prelude::GlobalStateContract;
use crate::prelude::value_cell::{ValueRef, ValueRefMut};
use crate::state::{StateContract, TState, UsizeState};
use crate::state::global_state::GlobalStateContainer;
use crate::state::state::State;
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct VecState<T> where T: StateContract {
    index_state: UsizeState,
    vec: TState<Vec<T>>,
}

impl<T: StateContract> VecState<T> {
    pub fn new<I: Into<UsizeState>, V: Into<TState<Vec<T>>>>(index: I, vec: V) -> VecState<T> {
        VecState {
            index_state: index.into(),
            vec: vec.into(),
        }
    }
}

impl<T: StateContract> State<T> for VecState<T> {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<T> {
        todo!()
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        todo!()
    }
}

impl<T: StateContract> Debug for VecState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::VecState")
            .field("value", self.deref())
            .field("index", &*self.index_state.value())
            .finish()
    }
}

impl<T: StateContract + 'static> Into<TState<T>> for VecState<T> {
    fn into(self) -> TState<T> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract + 'static> Into<TState<T>> for Box<VecState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}