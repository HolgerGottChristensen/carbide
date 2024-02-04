use std::marker::PhantomData;
use crate::environment::Environment;
use crate::state::{AnyReadState, AnyState, NewStateSync, ReadState, State, StateContract, ValueRef, ValueRefMut};

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

impl<T: StateContract, TState: State<T=T> + Clone + 'static> NewStateSync for LoggingState<T, TState> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.0.sync(env)
    }
}

impl<T: StateContract, TState: State<T=T> + Clone + 'static> AnyState for LoggingState<T, TState> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        // Get the current value
        let val = self.0.value().clone();

        // Clone self to get static lifetime
        let mut setter_self = self.clone();

        // Call set_value_dyn when ValueRefMut is dropped
        let setter = move |new| {
            setter_self.set_value_dyn(new);
        };

        ValueRefMut::TupleState(Some(Box::new(setter)), Some(val))
    }

    fn set_value_dyn(&mut self, val: T) {
        println!("Set value to: {:?}", &val);
        self.0.set_value(val);
    }
}