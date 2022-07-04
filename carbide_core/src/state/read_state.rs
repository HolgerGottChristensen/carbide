use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::state::state_sync::NewStateSync;
use crate::state::*;

use crate::state::util::value_cell::ValueRef;

/// The trait to implement for read-only state.
pub trait ReadState<T>: DynClone + NewStateSync + Debug
where
    T: StateContract,
{
    /// This retrieves a immutable reference to the value contained in the state.
    /// This type implements deref to get a reference to the actual value. The [`ValueRef`]
    /// should not be used directly.
    fn value(&self) -> ValueRef<T>;
}

dyn_clone::clone_trait_object!(<T: StateContract> ReadState<T>);
