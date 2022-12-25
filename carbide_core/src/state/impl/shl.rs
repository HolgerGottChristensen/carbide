use std::ops::Shl;

use crate::state::{Map2, RState, StateContract, WidgetState};
use crate::state::ReadWidgetState;

macro_rules! shl {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + Shl<U>, U: StateContract> Shl<$typ2> for $typ1
            where <T as Shl<U>>::Output: StateContract {

            type Output = RState<<T as Shl<U>>::Output>;

            fn shl(self, rhs: $typ2) -> Self::Output  {
                Map2::read_map(self.clone(), rhs.clone(), |val1: &T, val2: &U| {
                    val1.clone() << val2.clone()
                })
            }
        }
        )*
    };
}

shl!(
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
