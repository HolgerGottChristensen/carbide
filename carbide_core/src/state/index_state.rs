use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, Index, IndexMut};

use carbide_core::state::{AnyState, StateSync};

use crate::environment::{Environment, EnvironmentStack};
use crate::state::{AnyReadState, Fn2, Functor, IntoReadState, Map1, ReadState, RMap1, StateContract};
use crate::state::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};


/// # Index state
/// Index state is a general implementation that can take any state that is Index and IndexMut
/// with the same Idx and the same Output, and return a state containing that Output.
#[derive(Clone)]
pub struct IndexState<T, U, Idx, ST, SIdx>
where
    T: StateContract + Index<Idx, Output=U>,
    U: StateContract,
    Idx: StateContract,
    ST: State<T=T> + Clone + 'static,
    SIdx: ReadState<T=Idx> + Clone + 'static,
{
    /// The state that is evaluated whenever trying to get the index within the vec.
    index_state: SIdx,
    /// The state containing the vec.
    indexable_state: ST,
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
    phantom_idx: PhantomData<Idx>,
}

impl IndexState<Vec<()>, (), usize, Vec<()>, usize> {
    pub fn new<T, U, Idx, ST, SIdx>(indexable: ST, index: SIdx) -> IndexState<T, U, Idx, ST, SIdx> where
        T: StateContract + Index<Idx, Output=U>,
        U: StateContract,
        Idx: StateContract,
        ST: State<T=T> + Clone + 'static,
        SIdx: ReadState<T=Idx> + Clone + 'static
    {
        IndexState {
            indexable_state: indexable,
            index_state: index,
            phantom_t: Default::default(),
            phantom_u: Default::default(),
            phantom_idx: Default::default(),
        }
    }
}
/*
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
*/


impl<T, U, Idx, ST, SIdx> StateSync for IndexState<T, U, Idx, ST, SIdx>
where
    T: StateContract + Index<Idx, Output=U>,
    U: StateContract,
    Idx: StateContract,
    ST: State<T=T> + Clone + 'static,
    SIdx: ReadState<T=Idx> + Clone + 'static
{
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        let mut should_update = false;
        should_update |= self.index_state.sync(env);
        should_update |= self.indexable_state.sync(env);

        should_update
    }
}

impl<T, U, Idx, ST, SIdx> AnyReadState for IndexState<T, U, Idx, ST, SIdx>
where
    T: StateContract + Index<Idx, Output=U>,
    U: StateContract,
    Idx: StateContract,
    ST: State<T=T> + Clone + 'static,
    SIdx: ReadState<T=Idx> + Clone + 'static
{
    type T = U;
    fn value_dyn(&self) -> ValueRef<U> {
        let index = self.index_state.value().deref().clone();
        ValueRef::map(self.indexable_state.value(), |a| &a[index])
    }
}

// When the indexable collection implements IndexMut, the IndexState is additionally a mutable state.
impl<T, U, Idx, ST, SIdx> AnyState for IndexState<T, U, Idx, ST, SIdx>
where
    T: StateContract + Index<Idx, Output=U> + IndexMut<Idx, Output=U>,
    U: StateContract,
    Idx: StateContract,
    ST: State<T=T> + Clone + 'static,
    SIdx: ReadState<T=Idx> + Clone + 'static
{
    fn value_dyn_mut(&mut self) -> ValueRefMut<U> {
        let index = self.index_state.value().deref().clone();
        ValueRefMut::map(self.indexable_state.value_mut(), |a| &mut a[index])
    }

    fn set_value_dyn(&mut self, value: U) {
        *self.value_mut() = value;
    }
}

impl<T, U, Idx, ST, SIdx> Debug for IndexState<T, U, Idx, ST, SIdx>
where
    T: StateContract + Index<Idx, Output=U>,
    U: StateContract,
    Idx: StateContract,
    ST: State<T=T> + Clone + 'static,
    SIdx: ReadState<T=Idx> + Clone + 'static
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecState")
            .field("value", &*self.value())
            .field("index", &*self.index_state.value())
            .finish()
    }
}

impl<T: StateContract, V, U, Idx, ST, SIdx> Functor<T> for IndexState<V, U, Idx, ST, SIdx>
where
    IndexState<V, U, Idx, ST, SIdx>: IntoReadState<T>,
    V: StateContract + Index<Idx, Output=U>,
    U: StateContract,
    Idx: StateContract,
    ST: State<T=V> + Clone + 'static,
    SIdx: ReadState<T=Idx> + Clone + 'static
{
    // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
    type Output<G: StateContract, F: Fn2<T, G>> = RMap1<F, T, G, <IndexState<V, U, Idx, ST, SIdx> as IntoReadState<T>>::Output>;

    fn map<K: StateContract, F: Fn2<T, K>>(self, f: F) -> Self::Output<K, F> {
        Map1::read_map(self.into_read_state(), f)
    }
}

/*pub trait IndexableState<T, U, Idx>
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
}*/