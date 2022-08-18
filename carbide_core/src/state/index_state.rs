use carbide_core::prelude::NewStateSync;
use std::fmt::{Debug, Formatter};

use crate::prelude::Environment;
use crate::state::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;
use crate::state::{ReadState, StateContract, TState, UsizeState};

/// # Vector state
/// Vector state is a state mapping from a state of `Vec<T>` and a state of `usize` to a state of `T`.
/// Note: TODO
///
/// This state is ['Listenable'] and handles the subscriptions such that a change in either the
/// `usize` state or the `Vec<T>` state changes, the listener will receive a notification.
#[derive(Clone)]
pub struct IndexState<T>
where
    T: StateContract,
{
    /// The state that is evaluated whenever trying to get the index within the vec.
    index_state: TState<usize>,
    /// The state containing the vec.
    vec_state: TState<Vec<T>>,
}

impl<T: StateContract> IndexState<T> {
    pub fn new(vec: impl Into<TState<Vec<T>>>, index: impl Into<UsizeState>) -> TState<T> {
        Self::new_inner(vec, index).into()
    }

    fn new_inner(vec: impl Into<TState<Vec<T>>>, index: impl Into<UsizeState>) -> IndexState<T> {
        let vec_state = vec.into();
        let usize_state = index.into();

        IndexState {
            index_state: usize_state,
            vec_state: vec_state.into(),
        }
    }

    pub fn new2(vec: TState<Vec<T>>, index: UsizeState) -> TState<T> {
        Self::new_inner2(vec, index).into()
    }

    fn new_inner2(vec: TState<Vec<T>>, index: UsizeState) -> IndexState<T> {

        IndexState {
            index_state: index,
            vec_state: vec,
        }
    }
}

impl<T: StateContract> NewStateSync for IndexState<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut should_update = false;
        should_update |= self.index_state.sync(env);
        should_update |= self.vec_state.sync(env);

        should_update
    }
}

impl<T: StateContract> ReadState<T> for IndexState<T> {
    fn value(&self) -> ValueRef<T> {
        let index = *self.index_state.value();
        ValueRef::map(self.vec_state.value(), |a| &a[index])
    }
}

impl<T: StateContract> State<T> for IndexState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        let index = *self.index_state.value();
        ValueRefMut::map(self.vec_state.value_mut(), |a| &mut a[index])
    }

    fn set_value(&mut self, value: T) {
        *self.value_mut() = value;
    }
}

impl<T: StateContract> Debug for IndexState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecState")
            .field("value", &*self.value())
            .field("index", &*self.index_state.value())
            .finish()
    }
}

impl<T: StateContract> Into<TState<T>> for IndexState<T> {
    fn into(self) -> TState<T> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract> Into<TState<T>> for Box<IndexState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}
