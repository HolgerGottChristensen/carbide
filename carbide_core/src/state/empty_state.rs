use std::marker::PhantomData;
use carbide::state::{StateSync, ValueRefMut};
use crate::state::{AnyReadState, AnyState, StateContract, ValueRef};

#[derive(Debug, Copy, Clone)]
pub struct EmptyState<T>(PhantomData<T>);

impl<T: StateContract> StateSync for EmptyState<T> {}

impl<T: StateContract> AnyReadState for EmptyState<T> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        unimplemented!("You can not get a value from an empty state");
    }
}

impl<T: StateContract> AnyState for EmptyState<T> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Self::T> {
        unimplemented!("You can not get a mutable value from an empty state");
    }

    fn set_value_dyn(&mut self, _: Self::T) {
        unimplemented!("You can not set value to an empty state");
    }
}