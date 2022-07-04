use std::ops::Add;
use crate::state::{LocalState, Map2, ReadState, StateContract, ValueState, WidgetState};
use crate::state::RState;
use crate::state::ReadWidgetState;

macro_rules! add {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl<T: StateContract + Add<U>, U: StateContract> Add<$typ2> for $typ1
            where <T as Add<U>>::Output: StateContract {

            type Output = RState<<T as Add<U>>::Output>;

            fn add(self, rhs: $typ2) -> Self::Output  {
                Map2::read_map(self.clone(), rhs.clone(), |val1: &T, val2: &U| {
                    val1.clone() + val2.clone()
                })
            }
        }
        )*
    };
}

add!(
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
fn add_implemented_for_widget_state() {
    let a = LocalState::new(1);
    let b = LocalState::new(2);

    let c = a + b;
    assert_eq!(*c.value(), 3);

    let d = ValueState::new(1.0);
    let e = ValueState::new(2.0);

    let f = d + e;
    assert_eq!(*f.value(), 3.0);

    let g = LocalState::new(1);
    let h = LocalState::new(2);

    let i = &g + h.clone();
    let j = g.clone() + &h;
    let k = &g + &h;

    assert_eq!(*i.value(), 3);
    assert_eq!(*j.value(), 3);
    assert_eq!(*k.value(), 3);
}

#[test]
fn add_implemented_for_read_widget_state() {
    let a = LocalState::new(1).read_state();
    let b = LocalState::new(2).read_state();

    let c = a + b;
    assert_eq!(*c.value(), 3);

    let d = ValueState::new(1.0).read_state();
    let e = ValueState::new(2.0).read_state();

    let f = d + e;
    assert_eq!(*f.value(), 3.0);

    let g = LocalState::new(1).read_state();
    let h = LocalState::new(2).read_state();

    let i = &g + h.clone();
    let j = g.clone() + &h;
    let k = &g + &h;

    assert_eq!(*i.value(), 3);
    assert_eq!(*j.value(), 3);
    assert_eq!(*k.value(), 3);
}

#[test]
fn add_implemented_for_mixed_widget_state() {
    let a = LocalState::new(1).read_state();
    let b = LocalState::new(2);

    let c = a + b;
    assert_eq!(*c.value(), 3);

    let d = ValueState::new(1.0);
    let e = ValueState::new(2.0).read_state();

    let f = d + e;
    assert_eq!(*f.value(), 3.0);

    let g = LocalState::new(1).read_state();
    let h = LocalState::new(2);

    let i = &g + h.clone();
    let j = g.clone() + &h;
    let k = &g + &h;

    assert_eq!(*i.value(), 3);
    assert_eq!(*j.value(), 3);
    assert_eq!(*k.value(), 3);
}