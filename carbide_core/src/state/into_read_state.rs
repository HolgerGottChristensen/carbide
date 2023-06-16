use crate::state::{AnyReadState, ReadState, StateContract};

// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait IntoReadState<T>: Clone where T: StateContract {
    type Output: ReadState<T=T>;

    fn into_read_state(self) -> Self::Output;
}

/// Helper trait to allow us to implement distinct implementations of the same trait for
/// different associated types.
///
/// Based on: https://users.rust-lang.org/t/struggling-with-trait-impls/63535
///
/// Playground: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ec8e5b8920ccb55c4d552666fe7f1179
pub trait IntoReadStateHelper<T, U, B: StateContract>: Clone where T: AnyReadState<T=U> {
    type Output: ReadState<T=B>;

    fn into_read_state_helper(self) -> Self::Output;
}

// ---------------------------------------------------
//  Implementations
// ---------------------------------------------------

/// Default implementation that implements `IntoReadStateHelper` using the identity.
/// If something is a state of `U`, it can always be converted into a state of `U`.
impl<T, U: StateContract> IntoReadStateHelper<T, U, U> for T where T: AnyReadState<T=U> + Clone {
    type Output = T;

    fn into_read_state_helper(self) -> Self::Output {
        self
    }
}

/// A blanket implementation that implements `IntoReadState` for all things
/// that implement `IntoReadStateHelper`.
impl<T: AnyReadState<T=A>, A, B: StateContract> IntoReadState<B> for T where T: IntoReadStateHelper<T, A, B> {
    type Output = T::Output;

    fn into_read_state(self) -> Self::Output {
        self.into_read_state_helper()
    }
}