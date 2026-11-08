use std::collections::HashSet;
use std::hash::Hash;
use carbide::state::{AnyState, LocalState, State, StateContract, StateExtNew};

#[derive(Clone, Debug)]
pub(crate) enum ListSelection<T: StateContract> {
    Single(Box<dyn AnyState<T=Option<T>>>),
    Multi(Box<dyn AnyState<T=HashSet<T>>>),
}

impl<T: StateContract> From<LocalState<Option<T>>> for ListSelection<T> {
    fn from(val: LocalState<Option<T>>) -> Self {
        ListSelection::Single(val.as_dyn())
    }
}

impl<T: StateContract + Hash + Eq> From<LocalState<HashSet<T>>> for ListSelection<T> {
    fn from(val: LocalState<HashSet<T>>) -> Self {
        ListSelection::Multi(val.as_dyn())
    }
}

pub(crate) trait IntoSelection<T> where T: StateContract {
    fn convert(self) -> ListSelection<T>;
}

impl<T: StateContract, G: SelectableOf<T>, K> IntoSelection<T> for K where K: State<T=G> {
    fn convert(self) -> ListSelection<T> {
        G::convert(self)
    }
}

pub(crate) trait SelectableOf<T> where T: StateContract {
    fn convert<V: State<T=Self>>(value: V) -> ListSelection<T>;
}

impl<T: StateContract> SelectableOf<T> for Option<T> {
    fn convert<V: State<T=Self>>(value: V) -> ListSelection<T> {
        ListSelection::Single(value.as_dyn())
    }
}
impl<T: StateContract + Hash + Eq> SelectableOf<T> for HashSet<T> {
    fn convert<V: State<T=Self>>(value: V) -> ListSelection<T> {
        ListSelection::Multi(value.as_dyn())
    }
}