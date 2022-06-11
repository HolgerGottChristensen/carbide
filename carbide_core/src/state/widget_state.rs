use std::collections::HashSet;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Mul};

use dyn_clone::DynClone;
use carbide_core::prelude::{NewStateSync, ReadState};
use carbide_core::state::readonly::ReadWidgetState;
use carbide_core::state::RState;

use crate::prelude::Environment;
use crate::state::{Map1, Map2, Map3, StateContract, StateExt, TState, UsizeState, VecState};
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

    pub fn read_state(self) -> RState<T> {
        self.into()
    }
}



impl<T: StateContract> WidgetState<T> {
    /// Return a read-only state containing a boolean, which is true when self and other are
    /// equals.
    pub fn eq<U: StateContract + PartialEq<T>>(&self, other: impl Into<TState<U>>) -> RState<bool> {
        let other = other.into();
        Map2::read_map(self.clone(), other, |s1: &T, s2: &U| { s2 == s1 })
    }

    /// Return a read-only state containing a boolean, which is true when self and other are not
    /// equals.
    pub fn ne<U: StateContract + PartialEq<T>>(&self, other: impl Into<TState<U>>) -> RState<bool> {
        let other = other.into();
        Map2::read_map(self.clone(), other, |s1: &T, s2: &U| { s2 != s1 })
    }
}

impl<T: Debug> Debug for WidgetState<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl<T: Display> Display for WidgetState<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.0.fmt(fmt)
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
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.0.sync(env)
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