use carbide::state::{AnyState, LocalState, StateContract, StateExtNew, StateSync};
use std::collections::HashSet;
use std::hash::Hash;
use carbide::environment::EnvironmentStack;

#[derive(Clone, Debug)]
pub enum PickerSelection<T> where T: StateContract + PartialEq + Eq + Hash {
    Single(Box<dyn AnyState<T=T>>),
    Optional(Box<dyn AnyState<T=Option<T>>>),
    Multi(Box<dyn AnyState<T=HashSet<T>>>),
}

impl<T: StateContract + PartialEq + Eq + Hash> StateSync for PickerSelection<T> {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        match self {
            PickerSelection::Single(single) => single.sync(env),
            PickerSelection::Optional(optional) => optional.sync(env),
            PickerSelection::Multi(multi) => multi.sync(env),
        }
    }
}

impl<T: StateContract + PartialEq + Eq + Hash> Into<PickerSelection<T>> for LocalState<Option<T>> {
    fn into(self) -> PickerSelection<T> {
        PickerSelection::Optional(self.as_dyn())
    }
}

impl<T: StateContract + PartialEq + Eq + Hash> Into<PickerSelection<T>> for LocalState<HashSet<T>> {
    fn into(self) -> PickerSelection<T>  {
        PickerSelection::Multi(self.as_dyn())
    }
}

impl<T: StateContract + PartialEq + Eq + Hash> Into<PickerSelection<T>> for LocalState<T> {
    fn into(self) -> PickerSelection<T>  {
        PickerSelection::Single(self.as_dyn())
    }
}

impl<T: StateContract + PartialEq + Eq + Hash> PickerSelection<T> {
    pub fn to_type(&self) -> PickerSelectionType {
        match self {
            PickerSelection::Single(_) => PickerSelectionType::Single,
            PickerSelection::Optional(_) => PickerSelectionType::Optional,
            PickerSelection::Multi(_) => PickerSelectionType::Multi,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum PickerSelectionType {
    Optional,
    Single,
    Multi
}