use std::collections::HashSet;
use std::hash::Hash;
use crate::prelude::TState;
use crate::state::{Map1, Map2, RState, StateContract, WidgetState};

impl<T: StateContract + Eq + Hash> WidgetState<HashSet<T>> {

    /// Returns the number of elements in the set.
    pub fn len(&self) -> RState<usize> {
        Map1::read_map(self.clone(), |map: &HashSet<T>| {
            map.len()
        })
    }

    /// Returns a boolean state that is true if the set contains the value.
    pub fn contains(&self, contains: impl Into<TState<T>>) -> RState<bool> {
        Map2::read_map(self.clone(), contains.into(), |set: &HashSet<T>, value: &T| {
            set.contains(value)
        })
    }

    /// Returns a boolean state with true if the vec is empty.
    pub fn is_empty(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |set: &HashSet<T>| {
            set.is_empty()
        })
    }
}