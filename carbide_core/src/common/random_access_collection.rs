use carbide::identifiable::Identifiable;
use carbide::state::{IndexState, LocalState, ReadState, StateContract};
use std::ops::{Index, IndexMut, Range, RangeFrom};
use crate::state::AnyState;

/// A collection that can be accessed by an index, provides a start index, end index, and a way of
/// getting the next index.
pub trait RandomAccessCollection<T>: StateContract + 'static where T: StateContract + 'static {
    /// The index of the specific type
    type Idx: PartialOrd + StateContract + 'static;
    /// An iterator of all the indexes in the collection
    type Indices: IntoIterator<Item=Self::Idx> + StateContract + 'static;

    type Item<'a> where Self: 'a;

    /// Get value by index
    fn index(&self, index: Self::Idx) -> Self::Item<'_>;

    fn id(&self, index: Self::Idx) -> T::Id where T: Identifiable;

    /// Get the first index of the collection. If the collection is empty, this will return the
    /// same as the end index.
    fn start_index(&self) -> Self::Idx;
    /// Get an iterator of all the indices in the collection.
    fn indices(&self) -> Self::Indices;

    fn len(&self) -> usize;
    /// Get the end index of the collection. This will be the index one past the last valid index that
    /// can be used for getting a value.
    fn end_index(&self) -> Self::Idx;

    fn index_from_offset(&self, index: usize) -> Self::Idx;

    /// Provided an index, get the next index.
    fn next_index(&self, idx: Self::Idx) -> Self::Idx;
    fn prev_index(&self, idx: Self::Idx) -> Self::Idx;
}

impl<T: StateContract> RandomAccessCollection<T> for Vec<T> {
    type Idx = usize;
    type Indices = Range<usize>;
    type Item<'a> = &'a T;

    #[inline(always)]
    fn index(&self, index: Self::Idx) -> Self::Item<'_> {
        &self[index]
    }

    #[inline(always)]
    fn id(&self, index: Self::Idx) -> T::Id
    where
        T: Identifiable
    {
        self[index].id()
    }

    #[inline(always)]
    fn start_index(&self) -> Self::Idx {
        0
    }

    #[inline(always)]
    fn indices(&self) -> Self::Indices {
        0..self.len()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn end_index(&self) -> Self::Idx {
        self.len()
    }

    #[inline(always)]
    fn index_from_offset(&self, index: usize) -> Self::Idx {
        index
    }

    #[inline(always)]
    fn next_index(&self, idx: Self::Idx) -> Self::Idx {
        idx + 1
    }

    #[inline(always)]
    fn prev_index(&self, idx: Self::Idx) -> Self::Idx {
        idx.saturating_sub(1)
    }
}

impl RandomAccessCollection<u32> for Range<u32> {
    type Idx = u32;
    type Indices = Range<u32>;

    type Item<'a> = u32;

    #[inline(always)]
    fn index(&self, index: Self::Idx) -> u32 {
        index
    }

    #[inline(always)]
    fn id(&self, index: Self::Idx) -> <u32 as Identifiable>::Id
    {
        index
    }

    #[inline(always)]
    fn start_index(&self) -> Self::Idx {
        self.start
    }

    #[inline(always)]
    fn indices(&self) -> Self::Indices {
        self.start..self.end
    }

    #[inline(always)]
    fn len(&self) -> usize {
        ExactSizeIterator::len(self)
    }

    #[inline(always)]
    fn end_index(&self) -> Self::Idx {
        self.end
    }

    #[inline(always)]
    fn index_from_offset(&self, index: usize) -> Self::Idx {
        index as u32
    }

    #[inline(always)]
    fn next_index(&self, idx: Self::Idx) -> Self::Idx {
        idx + 1
    }

    #[inline(always)]
    fn prev_index(&self, idx: Self::Idx) -> Self::Idx {
        idx.saturating_sub(1)
    }
}

impl<T: StateContract + 'static, A: RandomAccessCollection<T>> RandomAccessCollection<T> for LocalState<A>
where
    A: Index<A::Idx, Output = T> + IndexMut<A::Idx, Output = T>,
    <A as RandomAccessCollection<T>>::Idx: ReadState<T=<A as RandomAccessCollection<T>>::Idx>
{
    type Idx = A::Idx;
    type Indices = A::Indices;
    type Item<'a> = Box<dyn AnyState<T=T>>;

    fn index(&self, index: Self::Idx) -> Self::Item<'_> {
        Box::new(IndexState::new(self.clone(), index))
    }

    fn id(&self, index: Self::Idx) -> T::Id
    where
        T: Identifiable
    {
        self.value().id(index)
    }

    fn start_index(&self) -> Self::Idx {
        self.value().start_index()
    }

    fn indices(&self) -> Self::Indices {
        self.value().indices()
    }

    fn len(&self) -> usize {
        self.value().len()
    }

    fn end_index(&self) -> Self::Idx {
        self.value().end_index()
    }

    fn index_from_offset(&self, index: usize) -> Self::Idx {
        self.value().index_from_offset(index)
    }

    fn next_index(&self, idx: Self::Idx) -> Self::Idx {
        self.value().next_index(idx)
    }

    fn prev_index(&self, idx: Self::Idx) -> Self::Idx {
        self.value().prev_index(idx)
    }
}