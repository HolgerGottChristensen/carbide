use std::fmt::{Debug, Formatter};

use crate::state::state_sync::StateSync;

use crate::environment::{Environment, EnvironmentStack};
use crate::state::{AnyReadState, ReadState, StateContract, AnyState, Functor, IntoReadState, RMap1, Fn2, Map1};
use crate::state::util::value_cell::{ValueCell, ValueRef, ValueRefMut};

#[derive(Clone)]
pub struct StaticState<T>
where
    T: StateContract,
{
    /// The shared state
    inner: &'static ValueCell<T>,
}

impl<T: StateContract> Copy for StaticState<T> {}

impl<T: StateContract> StaticState<T> {
    pub fn new(value: T) -> StaticState<T> {
        StaticState {
            inner: Box::leak(Box::new(ValueCell::new(value))),
        }
    }
}

impl<T: StateContract> StateSync for StaticState<T> {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        // TODO: find a smarter way to determine if static state has been updated.
        // I guess we can figuring it out by storing a frame number in the local state
        // and in the env, and then comparing and updating whenever this is called and set_value
        // is called.
        true
    }
}

impl<T: StateContract> AnyReadState for StaticState<T> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.inner.borrow()
    }
}

impl<T: StateContract> AnyState for StaticState<T> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        self.inner.borrow_mut()
    }

    fn set_value_dyn(&mut self, value: T) {
        *self.inner.borrow_mut() = value;
    }
}

impl<T: StateContract> Debug for StaticState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<T: StateContract, V: StateContract> Functor<T> for StaticState<V> where StaticState<V>: IntoReadState<T> {
    // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
    type Output<G: StateContract, F: Fn2<T, G>> = RMap1<F, T, G, <StaticState<V> as IntoReadState<T>>::Output>;

    fn map<U: StateContract, F: Fn2<T, U>>(self, f: F) -> Self::Output<U, F> {
        Map1::read_map(self.into_read_state(), f)
    }
}