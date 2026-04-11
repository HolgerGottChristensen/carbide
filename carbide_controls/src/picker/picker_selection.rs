use crate::identifiable::AnyIdentifiableWidget;
use carbide_core::state::{AnyState, LocalState, Map2, ReadStateExtNew, StateContract, StateExtNew, StateSync};
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use carbide::state::State;

pub trait PickerSelection<T>: StateSync + Clone + Debug + 'static where T: StateContract {
    fn selection_type(&self) -> PickerSelectionType;
    fn selection(&self, widget: &dyn AnyIdentifiableWidget<T=T>) -> Box<dyn AnyState<T=bool>>;
}

pub trait PickerSelectionKind<T> where T: StateContract + PartialEq {
    fn selection_type() -> PickerSelectionType;

    fn selection<V: State<T=Self>>(value: V, widget: &dyn AnyIdentifiableWidget<T=T>) -> Box<dyn AnyState<T=bool>>;
}

impl<T: StateContract + PartialEq> PickerSelectionKind<T> for T {
    fn selection_type() -> PickerSelectionType {
        PickerSelectionType::Single
    }

    fn selection<V: State<T=Self>>(value: V, widget: &dyn AnyIdentifiableWidget<T=T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            value,
            |value, selection| {
                value == selection
            },
            |new, value, mut selection| {
                if new {
                    *selection = value.clone();
                }
            }
        ).as_dyn()
    }
}

impl<T: StateContract + PartialEq> PickerSelectionKind<T> for Option<T> {
    fn selection_type() -> PickerSelectionType {
        PickerSelectionType::Optional
    }

    fn selection<V: State<T=Self>>(value: V, widget: &dyn AnyIdentifiableWidget<T=T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            value,
            |value, selection| {
                selection.as_ref().is_some_and(|x| x == value)
            },
            |new, value, mut selection| {
                if new {
                    *selection = Some(value.clone());
                } else {
                    *selection = None;
                }
            }
        ).as_dyn()
    }
}

impl<T: StateContract + PartialEq + Eq + Hash> PickerSelectionKind<T> for HashSet<T> {
    fn selection_type() -> PickerSelectionType {
        PickerSelectionType::Multi
    }

    fn selection<V: State<T=Self>>(value: V, widget: &dyn AnyIdentifiableWidget<T=T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            value,
            |value, selection| {
                selection.contains(value)
            },
            |new, value, mut selection| {
                if new {
                    selection.insert(value.clone());
                } else {
                    selection.remove(&*value);
                }
            }
        ).as_dyn()
    }
}

impl<T: StateContract + Ord> PickerSelectionKind<T> for BTreeSet<T> {
    fn selection_type() -> PickerSelectionType {
        PickerSelectionType::Multi
    }

    fn selection<V: State<T=Self>>(value: V, widget: &dyn AnyIdentifiableWidget<T=T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            value,
            |value, selection| {
                selection.contains(value)
            },
            |new, value, mut selection| {
                if new {
                    selection.insert(value.clone());
                } else {
                    selection.remove(&*value);
                }
            }
        ).as_dyn()
    }
}

impl<T: StateContract + PartialEq, G: PickerSelectionKind<T>, K> PickerSelection<T> for K where K: State<T=G> {
    fn selection_type(&self) -> PickerSelectionType {
        G::selection_type()
    }

    fn selection(&self, widget: &dyn AnyIdentifiableWidget<T=T>) -> Box<dyn AnyState<T=bool>> {
        G::selection(self.clone(), widget)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub(crate) enum PickerSelectionType {
    Optional,
    Single,
    Multi
}