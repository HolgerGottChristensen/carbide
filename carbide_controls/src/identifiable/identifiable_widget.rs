use carbide::state::{AnyReadState, ReadState, State, StateContract};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use dyn_clone::DynClone;
use carbide::widget::{AnySequence, AnyWidget, Delegate, ForEach, Identifiable, Sequence, Widget, WidgetExt, WidgetId, WidgetSync};
use carbide::widget::foreach_widget::ForEachWidget;

pub trait AnyIdentifiableWidget<T>: AnyWidget
where T: StateContract + PartialEq {
    fn identifier(&self) -> &dyn AnyReadState<T=T>;

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T>);

pub trait IdentifiableWidget<T>: AnyIdentifiableWidget<T> + WidgetExt + Clone where T: StateContract + PartialEq  {}

impl<T: StateContract + PartialEq, W> IdentifiableWidget<T> for W where W: AnyIdentifiableWidget<T> + WidgetExt + Clone {}

impl<T: StateContract + PartialEq> AnyIdentifiableWidget<T> for Box<dyn AnyIdentifiableWidget<T>> {
    fn identifier(&self) -> &dyn AnyReadState<T=T> {
        self.deref().identifier()
    }
}

impl<T: StateContract + PartialEq> WidgetExt for Box<dyn AnyIdentifiableWidget<T>> {

}

impl<T: StateContract + PartialEq> AnyWidget for Box<dyn AnyIdentifiableWidget<T>> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

impl<
    G: StateContract + PartialEq,
    T: StateContract,
    M: State<T=Vec<T>>,
    W: IdentifiableWidget<G>,
    U: Delegate<T, W>,
    I: ReadState<T=usize>> AnyIdentifiableWidget<G> for ForEach<T, M, U, W, I> {

    fn identifier(&self) -> &dyn AnyReadState<T=G> {
        todo!()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<G>)) {
        (self.children).foreach(f);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.children).foreach_mut(f);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.children).foreach_rev(f);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.children).foreach_direct(f);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.children).foreach_direct_rev(f);
    }
}

impl<
    G: StateContract + PartialEq,
    T: ?Sized + Identifiable + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: IdentifiableWidget<G>,
    D: carbide::widget::foreach_widget::Delegate<T, O>
> AnyIdentifiableWidget<G> for ForEachWidget<W, O, D, T> {

    fn identifier(&self) -> &dyn AnyReadState<T=G> {
        todo!()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<G>)) {
        (self.content).foreach(f);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.content).foreach_mut(f);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.content).foreach_rev(f);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.content).foreach_direct(f);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<G>)) {
        (self.content).foreach_direct_rev(f);
    }
}