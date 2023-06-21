use crate::state::{AnyReadState, ReadState, StateContract};

// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait IntoReadState<T>: Clone where T: StateContract {
    type Output: ReadState<T=T>;

    fn into_read_state(self) -> Self::Output;
}

pub trait ConvertIntoRead<T: StateContract>: StateContract {
    type Output<G: AnyReadState<T=Self> + Clone>: ReadState<T=T>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F>;
}

// ---------------------------------------------------
//  Implementations
// ---------------------------------------------------

impl<T: StateContract> ConvertIntoRead<T> for T {
    type Output<G: AnyReadState<T=Self> + Clone> = G;

    fn convert<F: AnyReadState<T=T> + Clone>(f: F) -> Self::Output<F> {
        f
    }
}

/// A blanket implementation that implements `IntoReadState` for all things
/// that implement `IntoReadStateHelper`.
impl<T: AnyReadState<T=A> + Clone, A: StateContract, B: StateContract> IntoReadState<B> for T where A: ConvertIntoRead<B> {
    type Output = A::Output<T>;

    fn into_read_state(self) -> Self::Output {
        A::convert(self)
    }
}