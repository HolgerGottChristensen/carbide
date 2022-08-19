use carbide_core::prelude::NewStateSync;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, Index, IndexMut};

use crate::prelude::Environment;
use crate::state::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;
use crate::state::{ReadState, RState, StateContract, TState, UsizeState};

/// # Index state
/// Index state is a general implementation that can take any state that is Index and IndexMut
/// with the same Idx and the same Output, and return a state containing that Output.
#[derive(Clone)]
pub struct IndexState<T, U, Idx>
where
    T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
    U: StateContract,
    Idx: StateContract,
{
    /// The state that is evaluated whenever trying to get the index within the vec.
    index_state: RState<Idx>,
    /// The state containing the vec.
    indexable_state: TState<T>,
}

impl<T, U, Idx> IndexState<T, U, Idx>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract,
{
    pub fn new(vec: impl Into<TState<T>>, index: impl Into<RState<Idx>>) -> TState<U> {
        Self::new_inner(vec, index).into()
    }

    fn new_inner(vec: impl Into<TState<T>>, index: impl Into<RState<Idx>>) -> IndexState<T, U, Idx> {
        let vec_state = vec.into();
        let usize_state = index.into();

        IndexState {
            index_state: usize_state,
            indexable_state: vec_state.into(),
        }
    }

    pub fn new2(vec: TState<T>, index: RState<Idx>) -> TState<U> {
        Self::new_inner2(vec, index).into()
    }

    fn new_inner2(vec: TState<T>, index: RState<Idx>) -> IndexState<T, U, Idx> {

        IndexState {
            index_state: index,
            indexable_state: vec,
        }
    }
}

impl<T, U, Idx> NewStateSync for IndexState<T, U, Idx>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract,
{
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut should_update = false;
        should_update |= self.index_state.sync(env);
        should_update |= self.indexable_state.sync(env);

        should_update
    }
}

impl<T, U, Idx> ReadState<U> for IndexState<T, U, Idx>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract
{
    fn value(&self) -> ValueRef<U> {
        let index = self.index_state.value().deref().clone();
        ValueRef::map(self.indexable_state.value(), |a| &a[index])
    }
}

impl<T, U, Idx> State<U> for IndexState<T, U, Idx>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract
{
    fn value_mut(&mut self) -> ValueRefMut<U> {
        let index = self.index_state.value().deref().clone();
        ValueRefMut::map(self.indexable_state.value_mut(), |a| &mut a[index])
    }

    fn set_value(&mut self, value: U) {
        *self.value_mut() = value;
    }
}

impl<T, U, Idx> Debug for IndexState<T, U, Idx>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecState")
            .field("value", &*self.value())
            .field("index", &*self.index_state.value())
            .finish()
    }
}

impl<T, U, Idx> Into<TState<U>> for IndexState<T, U, Idx>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract
{
    fn into(self) -> TState<U> {
        WidgetState::new(Box::new(self))
    }
}

impl<T, U, Idx> Into<TState<U>> for Box<IndexState<T, U, Idx>>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract
{
    fn into(self) -> TState<U> {
        WidgetState::new(self)
    }
}

pub trait IndexableState<T, U, Idx>
where T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
      U: StateContract,
      Idx: StateContract {
    fn index(&self, index: &TState<Idx>) -> TState<U>;
}

impl<T, U, Idx> IndexableState<T, U, Idx> for TState<T>
where T: Index<Idx, Output=U> + IndexMut<Idx, Output=U> + StateContract,
      U: StateContract,
      Idx: StateContract {
    fn index(&self, index: &TState<Idx>) -> TState<U> {
        IndexState::<T, U, Idx>::new(self.clone(), RState::new_from_read_write_state(index.clone()))
    }
}