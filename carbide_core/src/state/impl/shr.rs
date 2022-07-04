use crate::state::ReadWidgetState;
use crate::state::{Map2, RState, StateContract, WidgetState};
use std::ops::Shr;

macro_rules! shr {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + Shr<U>, U: StateContract> Shr<$typ2> for $typ1
            where <T as Shr<U>>::Output: StateContract {

            type Output = RState<<T as Shr<U>>::Output>;

            fn shr(self, rhs: $typ2) -> Self::Output  {
                Map2::read_map(self.clone(), rhs.clone(), |val1: &T, val2: &U| {
                    val1.clone() >> val2.clone()
                })
            }
        }
        )*
    };
}

shr!(
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
