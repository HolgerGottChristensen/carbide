use std::collections::HashSet;
use std::fmt;
use std::fmt::{Debug, Formatter};

use dyn_clone::DynClone;
use carbide_core::prelude::{NewStateSync, ReadState, Listenable, Id};
use carbide_core::state::readonly::ReadWidgetState;
use carbide_core::state::RState;

use crate::prelude::Environment;
use crate::state::{MapState, NewMapState, StateContract, StateExt, Listener, TState, UsizeState, VecState};
pub use crate::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};

/// # Widget state
/// This is a wrapper to make it easier to work with different kinds of read-write state.
/// It is commonly seen as ['TState'].
///
/// Its generic value is the type of state that will be received when calling ['value()']
/// It implements ['Clone'], ['Debug'] and is also listenable. When subscribing to this value
/// the listener is actually added to the inner state.
pub struct WidgetState<T>(Box<dyn State<T>>);

impl<T: StateContract> WidgetState<T> {
    pub fn new(item: Box<dyn State<T>>) -> WidgetState<T> {
        WidgetState(item)
    }

    pub fn to_boxed_state(self) -> Box<dyn State<T>> {
        self.0
    }
}

impl<T: StateContract> WidgetState<Vec<T>> {
    /// Returns a state that given an index will return a state containing the item at that index
    /// in the vector. It takes an UsizeState and will update the resulting state if either index
    /// or the vector changes.
    pub fn index(&self, index: UsizeState) -> TState<T> {
        VecState::new(self.clone(), index)
    }
}

impl<T: StateContract> WidgetState<Option<T>> {
    /// Allows calling is_some in the option. Returns a read-only boolean state. The reason it is
    /// read-only is because it would not be meaningful to set the state with a boolean and expect
    /// the original state to be changed.
    pub fn is_some(&self) -> RState<bool> {
        self.read_map(|t: &Option<T>| {
            t.is_some()
        })
    }

    pub fn is_none(&self) -> RState<bool> {
        self.read_map(|t: &Option<T>| {
            t.is_none()
        })
    }
}

impl<T: StateContract + Default + 'static> WidgetState<Option<T>> {
    pub fn unwrap_or_default(&self) -> TState<T> {
        NewMapState::<Option<T>, T>::new(
            self.clone(),
            |val| {
                val.clone().unwrap_or_default()
            },
            |new, old| {
                Some(new)
            }
        )
    }
}

impl<T: StateContract> WidgetState<HashSet<T>> {
    pub fn len(&self) -> RState<usize> {
        self.read_map(|map: &HashSet<T>| {
            map.len()
        })
    }
}

impl<T: StateContract> Debug for WidgetState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: StateContract> Clone for WidgetState<T> {
    fn clone(&self) -> Self {
        WidgetState(self.0.clone())
    }
}

impl<T: StateContract> Into<WidgetState<T>> for Box<dyn State<T>> {
    fn into(self) -> WidgetState<T> {
        WidgetState(self)
    }
}

impl<T: StateContract> Into<ReadWidgetState<T>> for WidgetState<T> {
    fn into(self) -> ReadWidgetState<T> {
        ReadWidgetState::ReadWriteState(self)
    }
}

impl<T: StateContract> NewStateSync for WidgetState<T> {
    fn sync(&mut self, env: &mut Environment) {
        self.0.sync(env)
    }
}

impl<T: StateContract> Listenable<T> for WidgetState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) -> Id {
        self.0.subscribe(subscriber)
    }

    fn unsubscribe(&self, id: &Id) {
        self.0.unsubscribe(id)
    }
}

impl<T: StateContract> ReadState<T> for WidgetState<T> {
    fn value(&self) -> ValueRef<T> {
        self.0.value()
    }
}

impl<T: StateContract> State<T> for WidgetState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.0.value_mut()
    }

    fn set_value(&mut self, value: T) {
        self.0.set_value(value)
    }

    fn notify(&self) {
        self.0.notify()
    }

    fn update_dependent(&mut self) {
        self.0.update_dependent()
    }
}

pub trait Map<FROM: StateContract, TO: StateContract>:
Fn(&FROM) -> TO + DynClone + 'static
{}

impl<T, FROM: StateContract, TO: StateContract> Map<FROM, TO> for T where
    T: Fn(&FROM) -> TO + DynClone + 'static
{}

dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> Map<FROM, TO>);