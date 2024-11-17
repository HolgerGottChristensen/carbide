mod idenfiable_widget;
mod selectable_sequence;
mod selectable_foreach;
mod identifiable_sequence;

use carbide::state::{AnyReadState, StateContract};
pub use idenfiable_widget::*;
pub use selectable_sequence::*;
pub use selectable_foreach::*;
pub use identifiable_sequence::*;

pub trait Identifiable<I: StateContract + PartialEq> {
    fn identifier(&self) -> Box<dyn AnyReadState<T=I>>;
    fn foreach_identifiable_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<I>));
}