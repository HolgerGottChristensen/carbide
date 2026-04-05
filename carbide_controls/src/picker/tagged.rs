use crate::identifiable::{AnyIdentifiableWidget};
use carbide::state::{AnyReadState, IntoReadState, ReadState, StateContract};
use carbide::widget::{AnySequence, CommonWidget, Empty, IntoWidget, Widget, WidgetId};
use carbide::ModifierWidgetImpl;
use std::fmt::Debug;
use carbide::identifiable::Identifiable;

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

impl<T: StateContract + PartialEq, C: Widget, S: ReadState<T=T>> AnyIdentifiableWidget<T> for Tagged<T, S, C> {
    fn identifier(&self) -> &dyn AnyReadState<T=T> {
        &self.tag
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
        todo!()//self.child.index(index)
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        todo!()//self.child.foreach(f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        todo!()//self.child.foreach_rev(f)
    }
}

impl<T: StateContract + PartialEq, C: Widget, S: ReadState<T=T>> Identifiable for Tagged<T, S, C> {
    type Id = WidgetId;

    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<T: StateContract + PartialEq, C: Widget, S: ReadState<T=T>> CommonWidget for Tagged<T, S, C> {
    ModifierWidgetImpl!(self, child: self.child);
}