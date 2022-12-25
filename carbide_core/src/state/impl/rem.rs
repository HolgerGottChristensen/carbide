use std::ops::Rem;

use crate::state::{Map2, RState, StateContract, WidgetState};
use crate::state::ReadWidgetState;

macro_rules! rem {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + Rem<U>, U: StateContract> Rem<$typ2> for $typ1
            where <T as Rem<U>>::Output: StateContract {

            type Output = RState<<T as Rem<U>>::Output>;

            fn rem(self, rhs: $typ2) -> Self::Output  {
                Map2::read_map(self.clone(), rhs.clone(), |val1: &T, val2: &U| {
                    val1.clone() % val2.clone()
                })
            }
        }
        )*
    };
}

rem!(
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
