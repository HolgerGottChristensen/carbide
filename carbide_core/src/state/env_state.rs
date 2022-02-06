use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use carbide_core::prelude::{NewStateSync, Listenable, Listener};

use crate::environment::Environment;
use crate::state::{ReadState, State, StateContract, TState};
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

/// Warning. This state can not be used to modify the environment, as it pulls the value out
/// of the environment at the time the state is captured. If the value is modified, the
/// modification is lost the next time the state is captured.
#[derive(Clone)]
pub struct EnvState<T>
    where
        T: StateContract,
{
    map: fn(env: &Environment) -> T,
    value: T,
}

impl<T: StateContract + Default> EnvState<T> {
    pub fn new(map: fn(env: &Environment) -> T) -> Self {
        EnvState {
            map,
            value: T::default(),
        }
    }
}

impl<T: StateContract> NewStateSync for EnvState<T> {
    fn sync(&mut self, env: &mut Environment) {
        self.value = (self.map)(env);
    }
}

impl<T: StateContract> Listenable<T> for EnvState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) {
        todo!()
    }
}

impl<T: StateContract> ReadState<T> for EnvState<T> {
    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }
}

impl<T: StateContract> State<T> for EnvState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: T) {
        self.value = value
    }

    fn notify(&self) {
        todo!()
    }
}

impl<T: StateContract> Debug for EnvState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::EnvState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract> Into<TState<T>> for Box<EnvState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}
