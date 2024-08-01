use crate::state::{AnyReadState, Map1, ReadState, ReadStateExtNew, RMap1, StateContract};

// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait IntoReadState<T>: Clone where T: StateContract {
    type Output: ReadState<T=T> + ReadStateExtNew<T>;

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


impl<T: StateContract> ConvertIntoRead<Option<T>> for T {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&T)->Option<T>, T, Option<T>, G>;

    fn convert<F: AnyReadState<T=T> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |c| {
            Some(c.clone())
        })
    }
}

#[macro_export]
macro_rules! impl_convert_into_read_cast {
    ($($typ_from: ty => $typ_to: ty),*) => {
        $(
        impl ConvertIntoRead<$typ_to> for $typ_from {
            type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&$typ_from)->$typ_to, $typ_from, $typ_to, G>;

            fn convert<F: AnyReadState<T=$typ_from> + Clone>(f: F) -> Self::Output<F> {
                Map1::read_map(f, |c| {
                    *c as $typ_to
                })
            }
        }
        )*
    };
}

impl_convert_into_read_cast!(i32 => u32, f64 => f32, f32 => f64);
