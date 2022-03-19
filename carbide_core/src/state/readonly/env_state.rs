use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use carbide_core::prelude::NewStateSync;

use crate::environment::Environment;
use crate::state::{ReadState, RState, StateContract};
use crate::state::readonly::ReadWidgetState;
use crate::state::util::value_cell::ValueRef;

/// # Environment state
/// EnvState is a read-only state that takes a function from the environment and returns a value.
/// This means you will not be able to set this state manually. The initial value of the state is
/// whatever default value is provided by the generic type `T`. If the field containing this state
/// is correctly marked with #\[state\] you should never in practice see this value as it is
/// updated whenever [NewStateSync::sync()] is called.
///
/// This state is [Listenable] and will notify all listeners whenever the value has changed. This
/// is also why `T` is required to implement [PartialEq].
///
/// [Clone]ing this value will result in differing states but they will still share the same list
/// of listeners. Note: This might change in the future.
#[derive(Clone)]
pub struct EnvState<T>
    where
        T: StateContract + PartialEq,
{
    /// The mapping function that take the environment and returns `T`, but you should make sure
    /// this function is not too expensive because it is called on every [NewStateSync::sync()].
    map: fn(env: &Environment) -> T,
    /// The value from the last mapping.
    value: T,
}

impl<T: StateContract + PartialEq + Default> EnvState<T> {
    /// Create a new environment state that takes the value out from the environment.
    ///
    /// ## Description
    /// * `map` - The mapping function that takes a reference to an env and returns a value.
    ///           Make sure this function is not to expensive, because it might be run often
    ///           depending on the use of the state.
    pub fn new(map: fn(env: &Environment) -> T) -> Self {
        EnvState {
            map,
            value: T::default(),
        }
    }
}

impl<T: StateContract + PartialEq> NewStateSync for EnvState<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let val = (self.map)(env);
        if val != self.value {
            self.value = val;
            true
        } else {
            false
        }
    }
}

impl<T: StateContract + PartialEq> ReadState<T> for EnvState<T> {
    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }
}

impl<T: StateContract + PartialEq> Debug for EnvState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract + PartialEq> Into<RState<T>> for Box<EnvState<T>> {
    fn into(self) -> RState<T> {
        ReadWidgetState::ReadState(self)
    }
}
