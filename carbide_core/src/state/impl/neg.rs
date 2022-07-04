use std::ops::Neg;
use crate::state::{Map1, StateContract, WidgetState};
use crate::state::ReadWidgetState;

impl<T: StateContract + Neg> Neg for WidgetState<T>
    where <T as Neg>::Output: StateContract {

    type Output = ReadWidgetState<<T as Neg>::Output>;

    fn neg(self) -> Self::Output  {
        Map1::read_map(self.clone(), |val1: &T| {
            -val1.clone()
        })
    }
}

impl<T: StateContract + Neg> Neg for &WidgetState<T>
    where <T as Neg>::Output: StateContract {

    type Output = ReadWidgetState<<T as Neg>::Output>;

    fn neg(self) -> Self::Output  {
        Map1::read_map(self.clone(), |val1: &T| {
            -val1.clone()
        })
    }
}