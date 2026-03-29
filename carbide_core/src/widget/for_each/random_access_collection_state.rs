// use std::ops::Index;
// use carbide::state::StateContract;
// use crate::state::{AnyReadState, IndexState, LocalState, ReadState};
// use crate::widget::foreach::random_access_collection::RandomAccessCollection;
//
// pub trait RandomAccessCollectionState<S, T> where S: RandomAccessCollection<T>, T: StateContract + 'static {
//     type Item<'a> where Self: 'a;
//
//     /// Get value by index
//     fn index_state(&self, index: S::Idx) -> Self::Item<'_>;
// }
//
// impl<T: StateContract + 'static> RandomAccessCollectionState<Vec<T>, T> for Vec<T> {
//     type Item<'a> = &'a T;
//
//     fn index_state(&self, index: <Vec<T> as RandomAccessCollection<T>>::Idx) -> Self::Item<'_> {
//         RandomAccessCollection::index(self, index)
//     }
// }
//
// impl<T: StateContract + 'static, A: RandomAccessCollection<T>> RandomAccessCollectionState<A, T> for LocalState<A>
// where
//     A: Index<A::Idx, Output = T>,
//     <A as RandomAccessCollection<T>>::Idx: ReadState<T=<A as RandomAccessCollection<T>>::Idx>
// {
//     type Item<'a> = IndexState<A, T, A::Idx, LocalState<A>, A::Idx>;
//
//     fn index_state(&self, index: A::Idx) -> Self::Item<'_> {
//         IndexState::new(self.clone(), index)
//     }
// }