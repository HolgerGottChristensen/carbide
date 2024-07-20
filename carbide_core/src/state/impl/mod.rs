/*pub mod hashset;
pub mod option;
pub mod vec;

pub mod add;
pub mod bitand;
pub mod bitor;
pub mod bitxor;
pub mod bool;
pub mod div;
pub mod mul;
pub mod neg;
pub mod not;
pub mod rem;
pub mod result;
pub mod shl;
pub mod shr;
pub mod sub;
pub mod eq;
pub mod and;
pub mod or;
pub mod ord;*/


use crate::state::StateContract;
use crate::state::{Map1, ReadState, RMap1};

pub trait ToStringState {
    type Output: ReadState<T=String>;

    fn to_string_state(&self) -> Self::Output;
}

impl<T, U: ToString + StateContract> ToStringState for T where T: ReadState<T=U> {
    type Output = RMap1<fn(&U)->String, U, String, T>;

    fn to_string_state(&self) -> Self::Output {
        Map1::read_map(self.clone(), |val| {
            val.to_string()
        })
    }
}

