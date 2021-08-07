use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::environment::environment::Environment;
use crate::prelude::value_cell::{ValueRef, ValueRefMut};
use crate::state::{State, StateContract, TState};
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct ValueState<T> where T: StateContract {
    value: T,
}

impl<T: StateContract> ValueState<T> {
    pub fn new(value: T) -> Self {
        ValueState {
            value
        }
    }
}

impl<T: StateContract> Deref for ValueState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: StateContract> DerefMut for ValueState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: StateContract> State<T> for ValueState<T> {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Borrow(&mut self.value)
    }
}

impl<T: StateContract> Debug for ValueState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::ValueState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract + 'static> Into<TState<T>> for Box<ValueState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}

/// This should implement into T state for pretty much all T.
impl<T: StateContract + 'static> From<T> for TState<T> {
    fn from(t: T) -> Self {
        WidgetState::new(Box::new(ValueState::new(t)))
    }
}