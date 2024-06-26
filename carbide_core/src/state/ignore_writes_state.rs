use std::marker::PhantomData;

use crate::environment::Environment;
use crate::state::{AnyReadState, AnyState, StateContract};
use crate::state::{NewStateSync, ReadState, ValueRef, ValueRefMut};

#[derive(Clone, Debug)]
pub struct IgnoreWritesState<T: StateContract, TState: ReadState<T=T> + Clone + 'static>(TState, PhantomData<T>);

impl IgnoreWritesState<(), ()> {
    pub fn new<T: StateContract, TState: ReadState<T=T> + Clone + 'static>(inner: TState) -> IgnoreWritesState<T, TState> {
        IgnoreWritesState(inner, PhantomData::default())
    }
}

impl<T: StateContract, TState: ReadState<T=T> + Clone + 'static> AnyReadState for IgnoreWritesState<T, TState> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.0.value()
    }
}

impl<T: StateContract, TState: ReadState<T=T> + Clone + 'static> NewStateSync for IgnoreWritesState<T, TState> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.0.sync(env)
    }
}

impl<T: StateContract, TState: ReadState<T=T> + Clone + 'static> AnyState for IgnoreWritesState<T, TState> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Read(self.value())
    }

    fn set_value_dyn(&mut self, _: T) {
        println!("WARNING: You are trying to set a state that is set to ignore writes");
    }
}

/*impl<T: StateContract, TState: ReadState<T=T> + Clone + 'static> IntoState<T> for IgnoreWritesState<T, TState> {
    type Output = Self;

    fn into_state(self) -> Self::Output {
        self
    }
}
*/