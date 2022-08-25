use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc};
use parking_lot::{RawRwLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::environment::Environment;
use crate::state::{NewStateSync, ReadState, State, StateContract, TState, ValueRef, ValueRefMut};

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

    /// Returns a new local state containing the value provided.
    /// Often you should use `new` when creating states, but this can be used to get the state
    /// within a box.
    fn new_raw(value: T) -> Box<Self> {
        Box::new(GlobalState {
            inner_value: Arc::new(RwLock::new(value)),
        })
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

impl<T: StateContract> ReadState<T> for GlobalState<T> {
    fn value(&self) -> ValueRef<T> {
        ValueRef::Locked(
            RwLockReadGuard::map(self.inner_value.read(), |a| a)
        )
    }
}

impl<T: StateContract> State<T> for GlobalState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Locked(
            RwLockWriteGuard::map(self.inner_value.write(), |a| a)
        )
    }

    fn set_value(&mut self, value: T) {
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
