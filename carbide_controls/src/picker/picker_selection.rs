use carbide::state::{AnyState, LocalState, StateContract, StateExtNew, StateSync};
use std::collections::HashSet;
use carbide::environment::EnvironmentStack;

#[derive(Clone, Debug)]
pub enum PickerSelection<T> where T: StateContract + PartialEq {
    Single(Box<dyn AnyState<T=T>>),
    Optional(Box<dyn AnyState<T=Option<T>>>),
    Multi(Box<dyn AnyState<T=HashSet<T>>>),
}

impl<T: StateContract + PartialEq> StateSync for PickerSelection<T> {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        match self {
            PickerSelection::Single(single) => single.sync(env),
            PickerSelection::Optional(optional) => optional.sync(env),
            PickerSelection::Multi(multi) => multi.sync(env),
        }
    }
}

impl<T: StateContract + PartialEq> Into<PickerSelection<T>> for LocalState<Option<T>> {
    fn into(self) -> PickerSelection<T> {
        PickerSelection::Optional(self.as_dyn())
    }
}

impl<T: StateContract + PartialEq> Into<PickerSelection<T>> for LocalState<HashSet<T>> {
    fn into(self) -> PickerSelection<T>  {
        PickerSelection::Multi(self.as_dyn())
    }
}

impl<T: StateContract + PartialEq> Into<PickerSelection<T>> for LocalState<T> {
    fn into(self) -> PickerSelection<T>  {
        PickerSelection::Single(self.as_dyn())
    }
}