use crate::state::{Map1, Map2, RState, StateContract, TState, VecState, WidgetState};

impl<T: StateContract> WidgetState<Vec<T>> {
    /// Returns a state that given an index will return a state containing the item at that index
    /// in the vector. It takes an UsizeState and will update the resulting state if either index
    /// or the vector changes.
    pub fn index(&self, index: TState<usize>) -> TState<T> {
        VecState::new(self.clone(), index)
    }

    /// Return the length of the vec as a state. This is read only, because setting the length
    /// of a Vec is not possible.
    pub fn len(&self) -> RState<usize> {
        Map1::read_map(self.clone(), |vec: &Vec<T>| vec.len())
    }

    /// Return the capacity of the Vec.
    pub fn capacity(&self) -> RState<usize> {
        Map1::read_map(self.clone(), |vec: &Vec<T>| vec.capacity())
    }

    /// Returns a boolean state with true if the vec is empty.
    pub fn is_empty(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |vec: &Vec<T>| vec.is_empty())
    }
}

impl<T: StateContract + PartialEq> TState<Vec<T>> {
    /// Return the capacity of the Vec.
    pub fn contains(&self, contains: impl Into<TState<T>>) -> RState<bool> {
        Map2::read_map(self.clone(), contains.into(), |vec: &Vec<T>, value: &T| {
            vec.contains(value)
        })
    }
}

impl<T: StateContract + Copy> WidgetState<Vec<T>> {
    /// Return the vec repeated n times.
    pub fn repeat(&self, n: impl Into<TState<usize>>) -> RState<Vec<T>> {
        Map2::read_map(self.clone(), n.into(), |vec: &Vec<T>, n: &usize| {
            vec.repeat(*n)
        })
    }
}
