use crate::state::ReadWidgetState;
use crate::state::{Map1, StateContract, WidgetState};
use std::ops::Neg;

impl<T: StateContract + Neg> Neg for WidgetState<T>
where
    <T as Neg>::Output: StateContract,
{
    type Output = ReadWidgetState<<T as Neg>::Output>;

    fn neg(self) -> Self::Output {
        Map1::read_map(self.clone(), |val1: &T| -val1.clone())
    }
}

impl<T: StateContract + Neg> Neg for &WidgetState<T>
where
    <T as Neg>::Output: StateContract,
{
    type Output = ReadWidgetState<<T as Neg>::Output>;

    fn neg(self) -> Self::Output {
        Map1::read_map(self.clone(), |val1: &T| -val1.clone())
    }
}
