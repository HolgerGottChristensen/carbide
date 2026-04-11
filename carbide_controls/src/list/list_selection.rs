use std::collections::HashSet;
use std::hash::Hash;
use carbide::state::{AnyState, LocalState, State, StateContract, StateExtNew};

#[derive(Clone, Debug)]
pub(crate) enum ListSelection<T: StateContract> {
    Single(Box<dyn AnyState<T=Option<T>>>),
    Multi(Box<dyn AnyState<T=HashSet<T>>>),
}

impl<T: StateContract> Into<ListSelection<T>> for LocalState<Option<T>> {
    fn into(self) -> ListSelection<T> {
        ListSelection::Single(self.as_dyn())
    }
}

impl<T: StateContract + Hash + Eq> Into<ListSelection<T>> for LocalState<HashSet<T>> {
    fn into(self) -> ListSelection<T> {
        ListSelection::Multi(self.as_dyn())
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