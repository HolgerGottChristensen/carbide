use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::prelude::Environment;
use crate::state::{InnerState, State, StateContract, TState};
use crate::state::value_cell::{ValueCell, ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct GlobalState<T> where T: StateContract {
    value: InnerState<T>,
}

impl<T: StateContract> GlobalState<T> {
    pub fn new(env: &Environment) -> Self {
        GlobalState {
            value: env.get_global_state::<T>()
        }
    }
}

impl<'a, T: StateContract> State<T> for GlobalState<T> {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<T> {
        self.value.borrow()
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.value.borrow_mut()
    }
}

impl<T: StateContract> Debug for GlobalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::GlobalState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<T: StateContract + 'static> Into<TState<T>> for Box<GlobalState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}