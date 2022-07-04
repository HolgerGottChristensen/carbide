use std::fmt::{Debug, Formatter};
use carbide_core::prelude::NewStateSync;

use crate::prelude::Environment;
use crate::state::{ReadState, StateContract, TState, UsizeState};
use crate::state::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

/// # Vector state
/// Vector state is a state mapping from a state of `Vec<T>` and a state of `usize` to a state of `T`.
/// Note: TODO
///
/// This state is ['Listenable'] and handles the subscriptions such that a change in either the
/// `usize` state or the `Vec<T>` state changes, the listener will receive a notification.
#[derive(Clone)]
pub struct VecState<T> where T: StateContract {
    /// The state that is evaluated whenever trying to get the index within the vec.
    index_state: TState<usize>,
    /// The state containing the vec.
    vec_state: TState<Vec<T>>,
}

impl<T: StateContract> VecState<T> {
    pub fn new(vec: impl Into<TState<Vec<T>>>, index: impl Into<UsizeState>) -> TState<T> {
        Self::new_inner(vec, index)
            .into()
    }

    fn new_inner(vec: impl Into<TState<Vec<T>>>, index: impl Into<UsizeState>) -> VecState<T> {
        let vec_state = vec.into();
        let usize_state = index.into();

        VecState {
            index_state: usize_state,
            vec_state: vec_state.into(),
        }
    }
}

impl<T: StateContract> NewStateSync for VecState<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut should_update = false;
        should_update |= self.index_state.sync(env);
        should_update |= self.vec_state.sync(env);

        should_update
    }
}

impl<T: StateContract> ReadState<T> for VecState<T> {
    fn value(&self) -> ValueRef<T> {
        let index = *self.index_state.value();
        ValueRef::map(self.vec_state.value(), |a| { &a[index] })
    }
}

impl<T: StateContract> State<T> for VecState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        let index = *self.index_state.value();
        ValueRefMut::map(self.vec_state.value_mut(), |a| &mut a[index])
    }

    fn set_value(&mut self, value: T) {
        *self.value_mut() = value;
    }
}

impl<T: StateContract> Debug for VecState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecState")
            .field("value", &*self.value())
            .field("index", &*self.index_state.value())
            .finish()
    }
}

impl<T: StateContract> Into<TState<T>> for VecState<T> {
    fn into(self) -> TState<T> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract> Into<TState<T>> for Box<VecState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}
