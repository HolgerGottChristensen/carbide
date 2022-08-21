use crate::state::{Map2, ReadWidgetState, RState, StateContract, TState, WidgetState};

pub trait StatePartialOrd<Rhs> {
    fn lt(self, other: Rhs) -> RState<bool>;

    fn le(self, other: Rhs) -> RState<bool>;

    fn gt(self, other: Rhs) -> RState<bool>;

    fn ge(self, other: Rhs) -> RState<bool>;
}

macro_rules! ord {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + PartialOrd<U>, U: StateContract> StatePartialOrd<$typ2> for $typ1 {
            fn lt(self, other: $typ2) -> RState<bool> {
                Map2::read_map(self.clone(), other.clone(), |s1: &T, s2: &U| s1 < s2)
            }

            fn le(self, other: $typ2) -> RState<bool> {
                Map2::read_map(self.clone(), other.clone(), |s1: &T, s2: &U| s1 <= s2)
            }

            fn gt(self, other: $typ2) -> RState<bool> {
                Map2::read_map(self.clone(), other.clone(), |s1: &T, s2: &U| s1 > s2)
            }

            fn ge(self, other: $typ2) -> RState<bool> {
                Map2::read_map(self.clone(), other.clone(), |s1: &T, s2: &U| s1 >= s2)
            }
        }
        )*
    };
}

ord!(
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
