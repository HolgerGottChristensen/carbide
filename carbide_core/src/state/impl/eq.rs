use crate::state::{Map2, ReadWidgetState, RState, StateContract, TState, WidgetState};

pub trait StatePartialEq<Rhs> {
    fn eq(self, other: Rhs) -> RState<bool>;

    fn ne(self, other: Rhs) -> RState<bool>;
}

macro_rules! eq {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + PartialEq<U>, U: StateContract> StatePartialEq<$typ2> for $typ1 {
            fn eq(self, other: $typ2) -> RState<bool> {
                Map2::read_map(self.clone(), other.clone(), |s1: &T, s2: &U| s1 == s2)
            }

            fn ne(self, other: $typ2) -> RState<bool> {
                Map2::read_map(self.clone(), other.clone(), |s1: &T, s2: &U| s1 != s2)
            }
        }
        )*
    };
}

eq!(
    WidgetState<T>, WidgetState<U>;
    WidgetState<T>, &WidgetState<U>;
    WidgetState<T>, ReadWidgetState<U>;
    WidgetState<T>, &ReadWidgetState<U>;

    &WidgetState<T>, WidgetState<U>;
    &WidgetState<T>, &WidgetState<U>;
    &WidgetState<T>, ReadWidgetState<U>;
    &WidgetState<T>, &ReadWidgetState<U>;

    ReadWidgetState<T>, WidgetState<U>;
    ReadWidgetState<T>, &WidgetState<U>;
    ReadWidgetState<T>, ReadWidgetState<U>;
    ReadWidgetState<T>, &ReadWidgetState<U>;

    &ReadWidgetState<T>, WidgetState<U>;
    &ReadWidgetState<T>, &WidgetState<U>;
    &ReadWidgetState<T>, ReadWidgetState<U>;
    &ReadWidgetState<T>, &ReadWidgetState<U>;
);
