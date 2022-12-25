use crate::environment::Environment;
use crate::state::{RState, State, StateContract, TState, WidgetState};
use crate::state::{NewStateSync, ReadState, ValueRef, ValueRefMut};

#[derive(Clone, Debug)]
pub struct IgnoreWritesState<T: StateContract>(RState<T>);

impl<T: StateContract> IgnoreWritesState<T> {
    pub fn new(inner: impl Into<RState<T>>) -> TState<T> {
        WidgetState::new(Box::new(Self(inner.into())))
    }
}

impl<T: StateContract> ReadState<T> for IgnoreWritesState<T> {
    fn value(&self) -> ValueRef<T> {
        self.0.value()
    }
}

impl<T: StateContract> NewStateSync for IgnoreWritesState<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.0.sync(env)
    }
}

impl<T: StateContract> State<T> for IgnoreWritesState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        panic!("Trying to get mutable value for a state that is readonly and ignoring writes.")
    }

    fn set_value(&mut self, _: T) {
        println!("WARNING: You are trying to set a state that is set to ignore writes");
    }
}
