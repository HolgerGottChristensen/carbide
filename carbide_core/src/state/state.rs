use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::prelude::Environment;
use crate::state::*;
use crate::state::readonly::ReadState;

use super::value_cell::{ValueRef, ValueRefMut};

pub trait State<T>: DynClone + Debug + ReadState<T>
    where
        T: StateContract,
{
    /*/// This should take the state from the environment to hold locally in the implementer.
    /// Other implementations could also take copies of global_state, and apply mappings to other
    /// states.
    /// This will always be the first thing called for the states of a widget when retrieving an
    /// event. This makes sure the local and other states are always up to date when recieving
    /// an event.
    fn capture_state(&mut self, env: &mut Environment) {}

    /// This releases local state from the widget back into the environment. This is called
    /// after the event has been processed in a widget, but before the children of the widget
    /// has is being processed. This makes sure the state is always available for the widget
    /// being processed.
    fn release_state(&mut self, env: &mut Environment) {}*/


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

    /// This function will notify all dependent states that this state has updated. This should be
    /// used for example by the dependents also updating their state.
    fn notify(&self);

    fn update_dependent(&mut self) {}
}

// impl<T: StateContract> ReadState<T> for dyn State<T> {
//     fn value(&self) -> ValueRef<T> {
//         self.value()
//     }
//
//     fn value_changed(&mut self) {
//         self.value_changed()
//     }
//
//     fn subscribe(&self, subscriber: Box<dyn Subscriber>) {
//         self.subscribe(subscriber)
//     }
// }

dyn_clone::clone_trait_object!(<T: StateContract> State<T>);
