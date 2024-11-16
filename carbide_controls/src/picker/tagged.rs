use crate::identifiable::Identifiable;
use carbide::state::{AnyReadState, IntoReadState, ReadState, ReadStateExtNew, StateContract, ValueRef};
use carbide::widget::{AnyWidget, CommonWidget, Empty, IntoWidget, Widget};
use carbide::ModifierWidgetImpl;
use std::fmt::Debug;

#[derive(Clone, Widget, Debug)]
pub struct Tagged<T, S, C> where T: StateContract + PartialEq, C: Widget, S: ReadState<T=T> {
    tag: S,
    child: C
}

impl Tagged<u32, u32, Empty> {
    pub fn new<S: ReadState<T=T>, T: StateContract + PartialEq, W: IntoWidget>(child: W, tag: S) -> Tagged<T, S, W::Output>{
        Tagged {
            tag,
            child: child.into_widget(),
        }
    }
}

impl<T: StateContract + PartialEq, C: Widget, S: ReadState<T=T>> CommonWidget for Tagged<T, S, C> {
    ModifierWidgetImpl!(self, child: self.child);
}

impl<T: StateContract + PartialEq, C: Widget, S: ReadState<T=T>> Identifiable<T> for Tagged<T, S, C> {
    fn identifier(&self) -> Box<dyn AnyReadState<T=T>> {
        self.tag.as_dyn_read()
    }
}