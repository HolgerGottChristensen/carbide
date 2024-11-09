use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use carbide_core::state::AnyState;
use carbide_core::state::state_sync::StateSync;

use crate::environment::{Environment, EnvironmentStack};
use crate::state::{AnyReadState, Fn2, Functor, InnerState, IntoReadState, Map1, ReadState, RMap1, StateContract};
use crate::state::util::value_cell::{ValueCell, ValueRef, ValueRefMut};

/// # Local state
/// The local state is used as a shared state between multiple widgets within the same widget tree.
/// When cloning this the inner state will be shared between the original and the clone.
/// The same is the case for the list of listeners.
///
/// Local state is [Listenable]. You are able to [Listenable::subscribe()] for notifications
/// whenever this state changes.
///
/// Local state does not need to do any updating when [StateSync::sync()] is called because
/// all state is stored directly within.
/// Also it does not depend on any other states and therefore the event can be ignored.
#[derive(Clone)]
pub struct LocalState<T>
    where
        T: StateContract,
{
    /// The shared state
    inner_value: InnerState<T>,
}

impl<T: StateContract> LocalState<T> {
    /// Returns a new local state containing the value provided.
    /// Returns the local state wrapped within a WidgetState.
    pub fn new(value: T) -> LocalState<T> {
        LocalState {
            inner_value: Rc::new(ValueCell::new(value)),
        }
    }
}

impl<T: StateContract> StateSync for LocalState<T> {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        // TODO: find a smarter way to determine if local state has been updated.
        // I guess we can figuring it out by storing a frame number in the local state
        // and in the env, and then comparing and updating whenever this is called and set_value
        // is called.
        true
    }
}

impl<T: StateContract> AnyReadState for LocalState<T> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.inner_value.borrow()
    }
}

impl<T: StateContract> AnyState for LocalState<T> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        self.inner_value.borrow_mut()
    }

    fn set_value_dyn(&mut self, value: T) {
        *self.inner_value.borrow_mut() = value;
    }
}

impl<T: StateContract> Debug for LocalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<T: StateContract, V: StateContract> Functor<T> for LocalState<V> where LocalState<V>: IntoReadState<T> {
    // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
    type Output<G: StateContract, F: Fn2<T, G>> = RMap1<F, T, G, <LocalState<V> as IntoReadState<T>>::Output>;

    fn map<U: StateContract, F: Fn2<T, U>>(self, f: F) -> Self::Output<U, F> {
        Map1::read_map(self.into_read_state(), f)
    }
}