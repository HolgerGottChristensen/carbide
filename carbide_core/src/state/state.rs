use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;
use carbide_core::environment::Environment;

use crate::state::*;
use crate::state::ReadState;
use crate::state::util::value_cell::ValueRefMut;

pub trait State: DynClone + Debug + ReadState {
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

    fn update_dependent(&mut self) {}
}

dyn_clone::clone_trait_object!(<T: StateContract> State<T=T>);

impl<T: StateContract> NewStateSync for Box<dyn State<T=T>> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.deref_mut().sync(env)
    }
}

impl<T: StateContract> ReadState for Box<dyn State<T=T>> {
    type T = T;
    fn value(&self) -> ValueRef<T> {
        self.deref().value()
    }
}

impl<T: StateContract> State for Box<dyn State<T=T>> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.deref_mut().value_mut()
    }

    fn set_value(&mut self, value: T) {
        self.deref_mut().set_value(value)
    }
}
