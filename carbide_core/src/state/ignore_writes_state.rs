use std::marker::PhantomData;
use carbide_core::state::IntoState;
use crate::environment::Environment;
use crate::state::{AnyReadState, AnyState, RState, State, StateContract, WidgetState};
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
        panic!("Trying to get mutable value for a state that is readonly and ignoring writes.")
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