use std::ops::BitAnd;
use crate::state::{Map2, StateContract, WidgetState, RState, LocalState, ValueState, ReadState};
use crate::state::readonly::ReadWidgetState;

macro_rules! bitand {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + BitAnd<U>, U: StateContract> BitAnd<$typ2> for $typ1
            where <T as BitAnd<U>>::Output: StateContract {

            type Output = RState<<T as BitAnd<U>>::Output>;

            fn bitand(self, rhs: $typ2) -> Self::Output  {
                Map2::read_map(self.clone(), rhs.clone(), |val1: &T, val2: &U| {
                    val1.clone() & val2.clone()
                })
            }
        }
        )*
    };
}

bitand!(
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

#[test]
fn bitand_implemented_for_widget_state() {
    let a = LocalState::new(1);
    let b = LocalState::new(2);

    let c = a & b;
    assert_eq!(*c.value(), 1 & 2);

    let d = ValueState::new(true);
    let e = ValueState::new(true);

    let f = d & e;
    assert_eq!(*f.value(), true & true);

    let g = LocalState::new(1);
    let h = LocalState::new(2);

    let i = &g & h.clone();
    let j = g.clone() & &h;
    let k = &g & &h;

    assert_eq!(*i.value(), 1 & 2);
    assert_eq!(*j.value(), 1 & 2);
    assert_eq!(*k.value(), 1 & 2);
}