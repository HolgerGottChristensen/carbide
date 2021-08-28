use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::prelude::Environment;
use crate::state::*;

use super::value_cell::{ValueRef, ValueRefMut};

pub trait State<T>: DynClone + Debug
    where
        T: StateContract,
{
    /// This should take the state from the environment to hold locally in the implementer.
    /// Other implementations could also take copies of global_state, and apply mappings to other
    /// states.
    /// This will always be the first thing called for the states of a widget when retrieving an
    /// event. This makes sure the local and other states are always up to date when recieving
    /// an event.
    fn capture_state(&mut self, env: &mut Environment);

    /// This releases local state from the widget back into the environment. This is called
    /// after the event has been processed in a widget, but before the children of the widget
    /// has is being processed. This makes sure the state is always available for the widget
    /// being processed.
    fn release_state(&mut self, env: &mut Environment);

    /// This retrieves a immutable reference to the value contained in the state.
    /// This type implements deref to get a reference to the actual value. The valueRef
    /// should not be used directly.
    fn value(&self) -> ValueRef<T>;

    /// This retrieves the value mutably. This is the entry point to changing a value in a state.
    /// This implements deref and deref_mut. Most state mutates the actual value in the state, but
    /// this is not guarantied, for example in state that contains a cloned version of another state.
    /// This is for example the case for MapOwnedState, EnvState and CloneState.
    /// If a ValueState is mutated, it will only affect that state, but not any clones of it.
    fn value_mut(&mut self) -> ValueRefMut<T>;
}

dyn_clone::clone_trait_object!(<T: StateContract> State<T>);
