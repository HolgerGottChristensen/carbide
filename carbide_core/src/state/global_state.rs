use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use carbide_core::state::AnyReadState;

use crate::environment::Environment;
use crate::state::{AnyState, NewStateSync, ReadState, StateContract, ValueRef, ValueRefMut};

/// # Local state
/// The local state is used as a shared state between multiple widgets within the same widget tree.
/// When cloning this the inner state will be shared between the original and the clone.
/// The same is the case for the list of listeners.
///
/// Local state is [Listenable]. You are able to [Listenable::subscribe()] for notifications
/// whenever this state changes.
///
/// Local state does not need to do any updating when [NewStateSync::sync()] is called because
/// all state is stored directly within.
/// Also it does not depend on any other states and therefore the event can be ignored.
#[derive(Clone)]
pub struct GlobalState<T>
    where
        T: StateContract,
{
    /// The shared state
    inner_value: Arc<RwLock<T>>,
}

impl<T: StateContract> GlobalState<T> {
    /// Returns a new local state containing the value provided.
    /// Returns the local state wrapped within a WidgetState.
    pub fn new(value: T) -> GlobalState<T> {
        GlobalState {
            inner_value: Arc::new(RwLock::new(value)),
        }
    }
}

impl<T: StateContract> NewStateSync for GlobalState<T> {
    fn sync(&mut self, _env: &mut Environment) -> bool {
        // TODO: find a smarter way to determine if local state has been updated.
        // I guess we can figuring it out by storing a frame number in the local state
        // and in the env, and then comparing and updating whenever this is called and set_value
        // is called.
        true
    }
}

impl<T: StateContract> AnyReadState for GlobalState<T> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        ValueRef::Locked(
            RwLockReadGuard::map(self.inner_value.read(), |a| a)
        )
    }
}

impl<T: StateContract> AnyState for GlobalState<T> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Locked(
            Some(RwLockWriteGuard::map(self.inner_value.write(), |a| a))
        )
    }

    fn set_value_dyn(&mut self, value: T) {
        *self.inner_value.write() = value;
    }
}

impl<T: StateContract> Debug for GlobalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalState")
            .field("value", &*self.value())
            .finish()
    }
}
