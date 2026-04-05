use carbide::state::{AnyReadState, ReadState, State, StateContract};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use dyn_clone::DynClone;
use carbide::identifiable::Identifiable;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::widget::{AnySequence, AnyWidget, Delegate, ForEach, Sequence, WidgetExt, WidgetId, WidgetProperties, WidgetSync};
use carbide::widget::foreach_widget::ForEachWidget;
use carbide::widget::properties::WidgetKindDynamic;

pub trait AnyIdentifiableWidget<T>: AnyWidget
where T: StateContract + PartialEq {
    fn identifier(&self) -> &dyn AnyReadState<T=T>;

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T>;
    fn child_count(&mut self) -> usize;

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>));
    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>));
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T>);

pub trait IdentifiableWidget<T>: AnyIdentifiableWidget<T> + WidgetExt + WidgetProperties + Clone where T: StateContract + PartialEq  {}

impl<T: StateContract + PartialEq, W> IdentifiableWidget<T> for W where W: AnyIdentifiableWidget<T> + WidgetExt + WidgetProperties + Clone {}

impl<T: StateContract + PartialEq> AnyIdentifiableWidget<T> for Box<dyn AnyIdentifiableWidget<T>> {
    fn identifier(&self) -> &dyn AnyReadState<T=T> {
        self.deref().identifier()
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
        AnyIdentifiableWidget::child(self.deref_mut(), index)
    }

    fn child_count(&mut self) -> usize {
        AnyIdentifiableWidget::child_count(self.deref_mut())
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        AnyIdentifiableWidget::foreach_child(self.deref_mut(), f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        AnyIdentifiableWidget::foreach_child_rev(self.deref_mut(), f)
    }
}

impl<T: StateContract + PartialEq> WidgetExt for Box<dyn AnyIdentifiableWidget<T>> {}
impl<T: StateContract + PartialEq> WidgetProperties for Box<dyn AnyIdentifiableWidget<T>> {
    type Kind = WidgetKindDynamic;
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
    T: StateContract + Identifiable,
    M: RandomAccessCollection<T>,
    W: IdentifiableWidget<G>,
    U: Delegate<M, T, W>> AnyIdentifiableWidget<G> for ForEach<T, M, U, W> {

    fn identifier(&self) -> &dyn AnyReadState<T=G> {
        todo!()
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<G> {
        todo!()
    }

    fn child_count(&mut self) -> usize {
        todo!()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<G>)) {
        todo!()//(self.children).foreach_mut(f);
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<G>)) {
        todo!()//(self.children).foreach_rev(f);
    }
}

impl<
    G: StateContract + PartialEq,
    T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: IdentifiableWidget<G>,
    D: carbide::widget::foreach_widget::Delegate<T, O>
> AnyIdentifiableWidget<G> for ForEachWidget<W, O, D, T> {

    fn identifier(&self) -> &dyn AnyReadState<T=G> {
        todo!()
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<G> {
        todo!()//self.content.index(index)
    }

    fn child_count(&mut self) -> usize {
        todo!()//AnySequence::<>::count(&mut self.content)
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<G>)) {
        todo!()//(self.content).foreach(f);
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<G>)) {
        todo!()//(self.content).foreach_rev(f);
    }
}