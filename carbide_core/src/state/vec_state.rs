use std::fmt::{Debug, Formatter};
use carbide_core::prelude::{NewStateSync, Listenable, Listener};

use crate::prelude::Environment;
use crate::state::{ReadState, StateContract, TState, UsizeState};
use crate::state::state::State;
use crate::state::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct VecState<T>
    where
        T: StateContract,
{
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

impl<T: StateContract> NewStateSync for VecState<T> {}

impl<T: StateContract> Listenable<T> for VecState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) {
        todo!()
    }
}

impl<T: StateContract> ReadState<T> for VecState<T> {
    fn value(&self) -> ValueRef<T> {
        todo!()
    }
}

impl<T: StateContract> State<T> for VecState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        todo!()
    }

    fn set_value(&mut self, value: T) {
        todo!()
    }

    fn notify(&self) {
        todo!()
    }
}

impl<T: StateContract> Debug for VecState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::VecState")
            .field("value", &*self.value())
            .field("index", &*self.index_state.value())
            .finish()
    }
}

impl<T: StateContract> Into<TState<T>> for VecState<T> {
    fn into(self) -> TState<T> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract> Into<TState<T>> for Box<VecState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}
