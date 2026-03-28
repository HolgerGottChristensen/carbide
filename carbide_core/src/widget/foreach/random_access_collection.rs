use std::ops::{Index, Range};
use carbide::state::{IndexState, LocalState, ReadState, StateContract};

/// A collection that can be accessed by an index, provides a start index, end index, and a way of
/// getting the next index.
pub trait RandomAccessCollection<T>: StateContract + 'static where T: StateContract + 'static {
    /// The index of the specific type
    type Idx: StateContract + 'static;
    /// An iterator of all the indexes in the collection
    type Indices: IntoIterator<Item=Self::Idx> + StateContract + 'static;

    type Item<'a> where Self: 'a;

    /// Get value by index
    fn index(&self, index: Self::Idx) -> Self::Item<'_>;

    /// Get the first index of the collection. If the collection is empty, this will return the
    /// same as the end index.
    fn start_index(&self) -> Self::Idx;
    /// Get an iterator of all the indices in the collection.
    fn indices(&self) -> Self::Indices;
    /// Get the end index of the collection. This will be the index one past the last valid index that
    /// can be used for getting a value.
    fn end_index(&self) -> Self::Idx;

    /// Provided an index, get the next index.
    fn next_index(&self, idx: Self::Idx) -> Self::Idx;
}

impl<T: StateContract> RandomAccessCollection<T> for Vec<T> {
    type Idx = usize;
    type Indices = Range<usize>;
    type Item<'a> = &'a T;

    fn index(&self, index: Self::Idx) -> Self::Item<'_> {
        &self[index]
    }

    fn start_index(&self) -> Self::Idx {
        0
    }

    fn indices(&self) -> Self::Indices {
        0..self.len()
    }

    fn end_index(&self) -> Self::Idx {
        self.len()
    }

    fn next_index(&self, idx: Self::Idx) -> Self::Idx {
        idx + 1
    }
}

impl RandomAccessCollection<u32> for Range<u32> {
    type Idx = u32;
    type Indices = Range<u32>;

    type Item<'a> = u32;

    fn index(&self, index: Self::Idx) -> u32 {
        index
    }

    fn start_index(&self) -> Self::Idx {
        self.start
    }

    fn indices(&self) -> Self::Indices {
        self.start..self.end
    }

    fn end_index(&self) -> Self::Idx {
        self.end
    }

    fn next_index(&self, idx: Self::Idx) -> Self::Idx {
        idx + 1
    }
}

impl<T: StateContract + 'static, A: RandomAccessCollection<T>> RandomAccessCollection<T> for LocalState<A>
where
    A: Index<A::Idx, Output = T>,
    <A as RandomAccessCollection<T>>::Idx: ReadState<T=<A as RandomAccessCollection<T>>::Idx>
{
    type Idx = A::Idx;
    type Indices = A::Indices;
    type Item<'a> = IndexState<A, T, A::Idx, LocalState<A>, A::Idx>;

    fn index(&self, index: Self::Idx) -> Self::Item<'_> {
        IndexState::new(self.clone(), index)
    }

    fn start_index(&self) -> Self::Idx {
        self.value().start_index()
    }

    fn indices(&self) -> Self::Indices {
        self.value().indices()
    }

    fn end_index(&self) -> Self::Idx {
        self.value().end_index()
    }

    fn next_index(&self, idx: Self::Idx) -> Self::Idx {
        self.value().next_index(idx)
    }
}