use std::ops::Div;
use crate::state::{Map2, StateContract, WidgetState, RState};
use crate::state::ReadWidgetState;

macro_rules! div {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + Div<U>, U: StateContract> Div<$typ2> for $typ1
            where <T as Div<U>>::Output: StateContract {

            type Output = RState<<T as Div<U>>::Output>;

            fn div(self, rhs: $typ2) -> Self::Output  {
                Map2::read_map(self.clone(), rhs.clone(), |val1: &T, val2: &U| {
                    val1.clone() / val2.clone()
                })
            }
        }
        )*
    };
}

div!(
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