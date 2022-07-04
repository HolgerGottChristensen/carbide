use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::state::*;
use crate::state::ReadState;

use crate::state::util::value_cell::ValueRefMut;

pub trait State<T>: DynClone + Debug + ReadState<T> where T: StateContract {
    /// This retrieves the value mutably. This is the entry point to changing a value in a state.
    /// This implements deref and deref_mut. Most state mutates the actual value in the state, but
    /// this is not guarantied, for example in state that contains a cloned version of another state.
    /// This is for example the case for MapOwnedState and EnvState.
    /// If a ValueState is mutated, it will only affect that state, but not any clones of it.
    /// After mutating the state, you should make sure to call [`State::notify()`]. This will
    /// make sure that all dependent states are notified that you have changed the state.
    fn value_mut(&mut self) -> ValueRefMut<T>;

    /// This is used to set the value of a state. Use this when you have state that might be mapped
    /// from the MapOwnedState. This makes sure that it is mapped all the way back to the original
    /// state. If you just change the value using value_mut, it might not be persistent and
    /// update problems might occur.
    fn set_value(&mut self, value: T);

    fn update_dependent(&mut self) {}
}

dyn_clone::clone_trait_object!(<T: StateContract> State<T>);
