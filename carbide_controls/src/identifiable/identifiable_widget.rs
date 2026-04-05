use carbide::state::{AnyReadState, ReadState, State, StateContract};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use dyn_clone::DynClone;
use carbide::identifiable::Identifiable;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::widget::{AnySequence, AnyWidget, Delegate, ForEach, Sequence, WidgetExt, WidgetId, WidgetProperties, WidgetSync};
use carbide::widget::foreach_widget::ForEachWidget;
use carbide::widget::properties::WidgetKindDynamic;

pub trait AnyIdentifiableWidget: AnyWidget {
    type T: StateContract + PartialEq;

    fn identifier(&self) -> &dyn AnyReadState<T=Self::T>;

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T=Self::T>;
    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=Self::T>));
    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=Self::T>));
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T=T>);

pub trait IdentifiableWidget: AnyIdentifiableWidget + WidgetExt + WidgetProperties + Clone {}

impl<W> IdentifiableWidget for W where W: AnyIdentifiableWidget + WidgetExt + WidgetProperties + Clone {}

impl<T: StateContract + PartialEq> AnyIdentifiableWidget for Box<dyn AnyIdentifiableWidget<T=T>> {
    type T = T;
    fn identifier(&self) -> &dyn AnyReadState<T=T> {
        self.deref().identifier()
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T=Self::T> {
        AnyIdentifiableWidget::child(self.deref_mut(), index)
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=Self::T>)) {
        AnyIdentifiableWidget::foreach_child(self.deref_mut(), f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=Self::T>)) {
        AnyIdentifiableWidget::foreach_child_rev(self.deref_mut(), f)
    }
}

impl<T: StateContract + PartialEq> WidgetExt for Box<dyn AnyIdentifiableWidget<T=T>> {}
impl<T: StateContract + PartialEq> WidgetProperties for Box<dyn AnyIdentifiableWidget<T=T>> {
    type Kind = WidgetKindDynamic;
}

impl<T: StateContract + PartialEq> AnyWidget for Box<dyn AnyIdentifiableWidget<T=T>> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

impl<
    G: StateContract + PartialEq,
    T: StateContract + Identifiable,
    M: RandomAccessCollection<T>,
    W: IdentifiableWidget<T=G>,
    U: Delegate<M, T, W>> AnyIdentifiableWidget for ForEach<T, M, U, W> {
    type T = G;
    fn identifier(&self) -> &dyn AnyReadState<T=G> {
        todo!()
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T=G> {
        todo!()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=G>)) {
        todo!()//(self.children).foreach_mut(f);
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=G>)) {
        todo!()//(self.children).foreach_rev(f);
    }
}

impl<
    G: StateContract + PartialEq,
    T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: IdentifiableWidget<T=G>,
    D: carbide::widget::foreach_widget::Delegate<T, O>
> AnyIdentifiableWidget for ForEachWidget<W, O, D, T> {
    type T = G;
    fn identifier(&self) -> &dyn AnyReadState<T=G> {
        todo!()
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T=G> {
        todo!()//self.content.index(index)
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=G>)) {
        todo!()//(self.content).foreach(f);
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T=G>)) {
        todo!()//(self.content).foreach_rev(f);
    }
}