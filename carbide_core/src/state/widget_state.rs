use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use dyn_clone::DynClone;

use carbide_core::state::ReadState;
use carbide_core::state::ReadWidgetState;
use carbide_core::state::RState;

use crate::environment::Environment;
use crate::state::{LocalState, NewStateSync, StateContract, ValueState};
use crate::state::global_state::GlobalState;
pub use crate::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};

/// # Widget state
/// This is a wrapper to make it easier to work with different kinds of read-write state.
/// It is commonly seen as ['TState'].
///
/// Its generic value is the type of state that will be received when calling ['value()']
/// It implements ['Clone'], ['Debug'] and is also listenable. When subscribing to this value
/// the listener is actually added to the inner state.
///
/// Below are the few enum cases that are able to be represented without requiring indirection.
/// Because of the need for indirection, we cannot create cases for Vec(VecState<T>), Flatten(Flatten<T>),
/// and so on, because they themselves contain WidgetState. Note: Interestingly, the rust compiler
/// will not even compile and through an error if both of these are added, causing compile times
/// to go through the roof(waited 5min and expect it to be an infinite loop, when trying to expand
/// the types).
///
/// FieldState and Map1-MapN states can not be represented, both because of the need for indirection
/// and because of the need for each enum case to contain generics, that are not shared across the
/// whole enum.
///
/// The fix for both of these is to use the Boxed enum case.
#[derive(Clone, Debug)]
pub enum WidgetState<T>
where
    T: StateContract,
{
    Value(ValueState<T>),
    Local(LocalState<T>),
    Global(GlobalState<T>),
    Boxed(Box<dyn State<T=T>>),
}

impl<T: StateContract> WidgetState<T> {
    pub fn new(item: Box<dyn State<T=T>>) -> WidgetState<T> {
        WidgetState::Boxed(item)
    }

    pub fn to_boxed_state(self) -> Box<dyn State<T=T>> {
        match self {
            WidgetState::Boxed(i) => i,
            WidgetState::Value(v) => Box::new(v),
            WidgetState::Local(v) => Box::new(v),
            WidgetState::Global(v) => Box::new(v),
        }
    }

    /*pub fn read_state(self) -> RState<T> {
        self.into()
    }*/
}

impl<T: Display + StateContract> Display for WidgetState<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            WidgetState::Boxed(i) => Display::fmt(&*i.value(), fmt),
            WidgetState::Value(v) => Display::fmt(&*v.value(), fmt),
            WidgetState::Local(v) => Display::fmt(&*v.value(), fmt),
            WidgetState::Global(v) => Display::fmt(&*v.value(), fmt),
        }
    }
}

/*impl<T: StateContract> Into<WidgetState<T>> for Box<dyn State<T>> {
    fn into(self) -> WidgetState<T> {
        WidgetState::new(self)
    }
}

impl<T: StateContract> Into<ReadWidgetState<T>> for WidgetState<T> {
    fn into(self) -> ReadWidgetState<T> {
        ReadWidgetState::ReadWriteState(self)
    }
}*/

impl<T: StateContract> NewStateSync for WidgetState<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self {
            WidgetState::Boxed(i) => i.sync(env),
            WidgetState::Value(v) => v.sync(env),
            WidgetState::Local(v) => v.sync(env),
            WidgetState::Global(v) => v.sync(env),
        }
    }
}

impl<T: StateContract> ReadState for WidgetState<T> {
    type T = T;

    fn value(&self) -> ValueRef<T> {
        match self {
            WidgetState::Boxed(i) => i.value(),
            WidgetState::Value(v) => v.value(),
            WidgetState::Local(v) => v.value(),
            WidgetState::Global(v) => v.value(),
        }
    }
}

impl<T: StateContract> State for WidgetState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        match self {
            WidgetState::Boxed(i) => i.value_mut(),
            WidgetState::Value(v) => v.value_mut(),
            WidgetState::Local(v) => v.value_mut(),
            WidgetState::Global(v) => v.value_mut(),
        }
    }

    fn set_value(&mut self, value: T) {
        match self {
            WidgetState::Boxed(i) => i.set_value(value),
            WidgetState::Value(v) => v.set_value(value),
            WidgetState::Local(v) => v.set_value(value),
            WidgetState::Global(v) => v.set_value(value),
        }
    }

    fn update_dependent(&mut self) {
        match self {
            WidgetState::Boxed(i) => i.update_dependent(),
            WidgetState::Value(v) => v.update_dependent(),
            WidgetState::Local(v) => v.update_dependent(),
            WidgetState::Global(v) => v.update_dependent(),
        }
    }
}

impl<T: StateContract> From<&WidgetState<T>> for WidgetState<T> {
    fn from(s: &WidgetState<T>) -> Self {
        s.clone()
    }
}

pub trait Map<FROM: StateContract, TO: StateContract>:
    Fn(&FROM) -> TO + DynClone + 'static
{
}

impl<T, FROM: StateContract, TO: StateContract> Map<FROM, TO> for T where
    T: Fn(&FROM) -> TO + DynClone + 'static
{
}

dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> Map<FROM, TO>);
