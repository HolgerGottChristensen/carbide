use std::marker::PhantomData;
use crate::environment::{Environment, EnvironmentStack};
use crate::state::{AnyReadState, AnyState, Fn2, Functor, IntoReadState, Map1, StateSync, RMap1, State, StateContract, ValueRef, ValueRefMut};

#[derive(Clone, Debug)]
pub struct LoggingState<T: StateContract, TState: State<T=T> + Clone + 'static>(TState, PhantomData<T>);

impl LoggingState<(), ()> {
    pub fn new<T: StateContract, TState: State<T=T> + Clone + 'static>(inner: TState) -> LoggingState<T, TState> {
        LoggingState(inner, PhantomData::default())
    }
}

impl<T: StateContract, TState: State<T=T> + Clone + 'static> AnyReadState for LoggingState<T, TState> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.0.value()
    }
}

impl<T: StateContract, TState: State<T=T> + Clone + 'static> StateSync for LoggingState<T, TState> {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        self.0.sync(env)
    }
}

impl<T: StateContract, TState: State<T=T> + Clone + 'static> AnyState for LoggingState<T, TState> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        println!("Retrieved value mut");
        self.0.value_mut()
    }

    fn set_value_dyn(&mut self, val: T) {
        println!("Set value to: {:?}", &val);
        self.0.set_value(val);
    }
}

impl<T: StateContract, V: StateContract, S: State<T=V> + Clone + 'static> Functor<T> for LoggingState<V, S> where LoggingState<V, S>: IntoReadState<T> {
    // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
    type Output<G: StateContract, F: Fn2<T, G>> = RMap1<F, T, G, <LoggingState<V, S> as IntoReadState<T>>::Output>;

    fn map<U: StateContract, F: Fn2<T, U>>(self, f: F) -> Self::Output<U, F> {
        Map1::read_map(self.into_read_state(), f)
    }
}