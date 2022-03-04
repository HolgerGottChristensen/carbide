use carbide_core::environment::Environment;
use carbide_core::prelude::{Id, Listenable, Listener, NewStateSync, ReadState, ValueRef, ValueRefMut};
use crate::state::{RState, State, StateContract, TState, WidgetState};

#[derive(Clone, Debug)]
pub struct IgnoreWritesState<T: StateContract>(RState<T>);

impl<T: StateContract> IgnoreWritesState<T> {
    pub fn new(inner: RState<T>) -> TState<T> {
        WidgetState::new(Box::new(Self(inner)))
    }
}

impl<T: StateContract> ReadState<T> for IgnoreWritesState<T> {
    fn value(&self) -> ValueRef<T> {
        self.0.value()
    }
}

impl<T: StateContract> NewStateSync for IgnoreWritesState<T> {
    fn sync(&mut self, env: &mut Environment) {
        self.0.sync(env)
    }
}

impl<T: StateContract> Listenable<T> for IgnoreWritesState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) -> Id {
        self.0.subscribe(subscriber)
    }

    fn unsubscribe(&self, id: &Id) {
        self.0.unsubscribe(id)
    }
}

impl<T: StateContract> State<T> for IgnoreWritesState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        panic!("Trying to get mutable value for a state that is readonly and ignoring writes.")
    }

    fn set_value(&mut self, _: T) {}

    fn notify(&self) {}
}