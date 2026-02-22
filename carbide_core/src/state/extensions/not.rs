use std::ops::Not;
use crate::state::{AnyReadState, Functor, Map1, RMap1};

impl Not for Box<dyn AnyReadState<T=bool>> {
    type Output = RMap1<fn(&bool)->bool, bool, bool, Box<dyn AnyReadState<T=bool>>>;

    fn not(self) -> Self::Output {
        Map1::read_map(self, |a| !*a)
    }
}