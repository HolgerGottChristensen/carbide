mod idenfiable_widget;

use carbide::state::{AnyReadState, StateContract, ValueRef};
pub use idenfiable_widget::*;

pub trait Identifiable<I: StateContract + PartialEq> {
    fn identifier(&self) -> Box<dyn AnyReadState<T=I>>;
}