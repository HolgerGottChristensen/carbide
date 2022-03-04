use std::fmt::{Debug, Formatter};
use carbide_core::prelude::{NewStateSync, Listenable, Listener, Id};

use crate::prelude::Environment;
use crate::state::{InnerState, ReadState, State, StateContract, TState};
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct GlobalState<T>
    where
        T: StateContract,
{
    value: InnerState<T>,
}

impl<T: StateContract> GlobalState<T> {
    pub fn new(env: &Environment) -> Self {
        GlobalState {
            value: env.get_global_state::<T>(),
        }
    }
}

impl<T: StateContract> NewStateSync for GlobalState<T> {}

impl<T: StateContract> Listenable<T> for GlobalState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) -> Id {
        todo!()
    }

    fn unsubscribe(&self, id: &Id) {
        todo!()
    }
}

impl<T: StateContract> ReadState<T> for GlobalState<T> {
    fn value(&self) -> ValueRef<T> {
        self.value.borrow()
    }
}

impl<T: StateContract> State<T> for GlobalState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.value.borrow_mut()
    }

    fn set_value(&mut self, value: T) {
        *self.value.borrow_mut() = value;
    }

    fn notify(&self) {
        todo!()
    }
}

impl<T: StateContract> Debug for GlobalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::GlobalState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<T: StateContract> Into<TState<T>> for Box<GlobalState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}
