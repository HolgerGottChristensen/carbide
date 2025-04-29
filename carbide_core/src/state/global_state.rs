use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::state::AnyReadState;

use crate::environment::{Environment};
use crate::state::{AnyState, Fn2, Functor, IntoReadState, Map1, StateSync, ReadState, RMap1, StateContract, ValueRef, ValueRefMut};

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

impl<T: StateContract> StateSync for GlobalState<T> {
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

impl<T: StateContract, V: StateContract> Functor<T> for GlobalState<V> where GlobalState<V>: IntoReadState<T> {
    type Output<G: StateContract, F: Fn2<T, G>> = RMap1<F, T, G, <GlobalState<V> as IntoReadState<T>>::Output>;

    fn map<U: StateContract, F: Fn2<T, U>>(self, f: F) -> Self::Output<U, F> {
        Map1::read_map(self.into_read_state(), f)
    }
}
