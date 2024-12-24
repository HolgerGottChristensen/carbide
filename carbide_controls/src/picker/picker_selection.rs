use crate::identifiable::AnyIdentifiableWidget;
use carbide_core::state::{AnyState, LocalState, Map2, ReadStateExtNew, StateContract, StateExtNew, StateSync};
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

pub trait PickerSelection<T>: StateSync + Clone + Debug + 'static where T: StateContract {
    fn selection_type(&self) -> PickerSelectionType;
    fn selection(&self, widget: &dyn AnyIdentifiableWidget<T>) -> Box<dyn AnyState<T=bool>>;
}

impl<T: StateContract + PartialEq> PickerSelection<T> for LocalState<T> {
    fn selection_type(&self) -> PickerSelectionType {
        PickerSelectionType::Single
    }

    fn selection(&self, widget: &dyn AnyIdentifiableWidget<T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            self.clone(),
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

impl<T: StateContract + PartialEq> PickerSelection<T> for LocalState<Option<T>> {
    fn selection_type(&self) -> PickerSelectionType {
        PickerSelectionType::Optional
    }

    fn selection(&self, widget: &dyn AnyIdentifiableWidget<T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            self.clone(),
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

impl<T: StateContract + PartialEq + Eq + Hash> PickerSelection<T> for LocalState<HashSet<T>> {
    fn selection_type(&self) -> PickerSelectionType {
        PickerSelectionType::Multi
    }

    fn selection(&self, widget: &dyn AnyIdentifiableWidget<T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            self.clone(),
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

impl<T: StateContract + Ord> PickerSelection<T> for LocalState<BTreeSet<T>> {
    fn selection_type(&self) -> PickerSelectionType {
        PickerSelectionType::Multi
    }

    fn selection(&self, widget: &dyn AnyIdentifiableWidget<T>) -> Box<dyn AnyState<T=bool>> {
        Map2::map(
            widget.identifier().boxed().ignore_writes(),
            self.clone(),
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

#[derive(Clone, Debug, Copy, PartialEq)]
pub(crate) enum PickerSelectionType {
    Optional,
    Single,
    Multi
}