use std::ops::Not;
use crate::state::{Map1, StateContract, WidgetState};
use crate::state::ReadWidgetState;

impl<T: StateContract + Not> Not for WidgetState<T>
    where <T as Not>::Output: StateContract {

    type Output = ReadWidgetState<<T as Not>::Output>;

    fn not(self) -> Self::Output  {
        Map1::read_map(self.clone(), |val1: &T| {
            !val1.clone()
        })
    }
}

impl<T: StateContract + Not> Not for &WidgetState<T>
    where <T as Not>::Output: StateContract {

    type Output = ReadWidgetState<<T as Not>::Output>;

    fn not(self) -> Self::Output  {
        Map1::read_map(self.clone(), |val1: &T| {
            !val1.clone()
        })
    }
}