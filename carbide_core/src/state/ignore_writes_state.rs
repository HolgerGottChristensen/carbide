use std::marker::PhantomData;

use crate::environment::{Environment};
use crate::state::{AnyReadState, AnyState, Fn2, Functor, IntoReadState, Map1, RMap1, StateContract};
use crate::state::{StateSync, ReadState, ValueRef, ValueRefMut};

#[derive(Clone, Debug)]
pub struct IgnoreWritesState<T: StateContract, TState: ReadState<T=T> + 'static>(TState, PhantomData<T>);

impl IgnoreWritesState<(), ()> {
    pub fn new<T: StateContract, TState: ReadState<T=T> + Clone + 'static>(inner: TState) -> IgnoreWritesState<T, TState> {
        IgnoreWritesState(inner, PhantomData::default())
    }
}

impl<T: StateContract, TState: ReadState<T=T> + 'static> IgnoreWritesState<T, TState> {
    pub fn inner(self) -> TState {
        self.0
    }
}

impl<T: StateContract, TState: ReadState<T=T> + Clone + 'static> AnyReadState for IgnoreWritesState<T, TState> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.0.value()
    }
}

impl<T: StateContract, TState: ReadState<T=T> + Clone + 'static> StateSync for IgnoreWritesState<T, TState> {
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

impl<T: StateContract, V: StateContract, TState: ReadState<T=V> + Clone + 'static> Functor<T> for IgnoreWritesState<V, TState> where IgnoreWritesState<V, TState>: IntoReadState<T> {
    // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
    type Output<G: StateContract, F: Fn2<T, G>> = RMap1<F, T, G, <IgnoreWritesState<V, TState> as IntoReadState<T>>::Output>;

    fn map<U: StateContract, F: Fn2<T, U>>(self, f: F) -> Self::Output<U, F> {
        Map1::read_map(self.into_read_state(), f)
    }
}

/*impl<T: StateContract, TState: ReadState<T=T> + Clone + 'static> IntoState<T> for IgnoreWritesState<T, TState> {
    type Output = Self;

    fn into_state(self) -> Self::Output {
        self
    }
}
*/