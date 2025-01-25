
use std::ops::{Deref, DerefMut};
use dyn_clone::clone_box;
use crate::environment::Environment;
use crate::state::*;
use crate::state::ReadState;
use crate::state::util::value_cell::ValueRefMut;

// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait State: ReadState + AnyState + IntoState<Self::T> + private::Sealed {
    /// This retrieves the value mutably. This is the entry point to changing a value in a state.
    /// This implements deref and deref_mut. Most state mutates the actual value in the state, but
    /// this is not guarantied, for example in state that contains a cloned version of another state.
    /// This is for example the case for MapOwnedState and EnvState.
    /// If a ValueState is mutated, it will only affect that state, but not any clones of it.
    /// After mutating the state, you should make sure to call [`State::notify()`]. This will
    /// make sure that all dependent states are notified that you have changed the state.
    fn value_mut(&mut self) -> ValueRefMut<Self::T>;

    /// This is used to set the value of a state. Use this when you have state that might be mapped
    /// from the MapOwnedState. This makes sure that it is mapped all the way back to the original
    /// state. If you just change the value using value_mut, it might not be persistent and
    /// update problems might occur.
    fn set_value(&mut self, value: Self::T);
}

pub trait AnyState: AnyReadState {
    /// This retrieves the value mutably. This is the entry point to changing a value in a state.
    /// This implements deref and deref_mut. Most state mutates the actual value in the state, but
    /// this is not guarantied, for example in state that contains a cloned version of another state.
    /// This is for example the case for MapOwnedState and EnvState.
    /// If a ValueState is mutated, it will only affect that state, but not any clones of it.
    /// After mutating the state, you should make sure to call [`State::notify()`]. This will
    /// make sure that all dependent states are notified that you have changed the state.
    fn value_dyn_mut(&mut self) -> ValueRefMut<Self::T>;

    /// This is used to set the value of a state. Use this when you have state that might be mapped
    /// from the MapOwnedState. This makes sure that it is mapped all the way back to the original
    /// state. If you just change the value using value_mut, it might not be persistent and
    /// update problems might occur.
    fn set_value_dyn(&mut self, value: Self::T);
}

impl<T: StateContract> dyn AnyState<T=T> {
    pub fn boxed(&self) -> Box<dyn AnyState<T=T>> {
        clone_box(self)
    }
}

// ---------------------------------------------------
//  Implementations
// ---------------------------------------------------
impl<T: StateContract> StateSync for Box<dyn AnyState<T=T>> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.deref_mut().sync(env)
    }
}

impl<T: StateContract> AnyReadState for Box<dyn AnyState<T=T>> {
    type T = T;

    fn value_dyn(&self) -> ValueRef<Self::T> {
        self.deref().value_dyn()
    }
}

impl<T: StateContract> AnyState for Box<dyn AnyState<T=T>> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Self::T> {
        self.deref_mut().value_dyn_mut()
    }

    fn set_value_dyn(&mut self, value: Self::T) {
        self.deref_mut().set_value_dyn(value)
    }
}

impl<T> State for T where T: AnyState + Clone + IntoState<Self::T> {
    fn value_mut(&mut self) -> ValueRefMut<Self::T> {
        self.value_dyn_mut()
    }

    fn set_value(&mut self, value: Self::T) {
        self.set_value_dyn(value)
    }
}

dyn_clone::clone_trait_object!(<T: StateContract> AnyState<T=T>);

// ---------------------------------------------------
//  Utility
// ---------------------------------------------------
mod private {
    use crate::state::AnyState;

    // This disallows implementing State manually, and requires something to implement AnyState
    // to implement State.
    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyState {}
}
