use carbide::state::{AnyReadState, ReadState, State, StateContract};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use carbide::widget::{AnyWidget, Delegate, ForEach, Sequence, Widget, WidgetExt, WidgetId};

pub trait AnyIdentifiableWidget<T>: AnyWidget
where T: StateContract + PartialEq {
    fn identifier(&self) -> &dyn AnyReadState<T=T>;
    fn as_widget(&self) -> &dyn AnyWidget;

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {}
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T>);

pub trait IdentifiableWidget<T>: AnyIdentifiableWidget<T> + WidgetExt + Clone where T: StateContract + PartialEq  {}

impl<T: StateContract + PartialEq, W> IdentifiableWidget<T> for W where W: AnyIdentifiableWidget<T> + WidgetExt + Clone {}

impl<T: StateContract + PartialEq, T1: IdentifiableWidget<T>, T2: IdentifiableWidget<T>> Sequence<dyn AnyIdentifiableWidget<T>> for (T1, T2) {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<T>)) {
        let (W1, W2) = self;
        if W1.is_ignore() {} else if W1.is_proxy() {
            AnyIdentifiableWidget::foreach_child(W1, f);
        } else {
            f(W1);
        }
        if W2.is_ignore() {} else if W2.is_proxy() {
            AnyIdentifiableWidget::foreach_child(W2, f);
        } else {
            f(W2);
        }
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        let (W1, W2) = self;
        if W1.is_ignore() {} else if W1.is_proxy() {
            AnyIdentifiableWidget::foreach_child_mut(W1, f);
        } else {
            f(W1);
        }
        if W2.is_ignore() {} else if W2.is_proxy() {
            AnyIdentifiableWidget::foreach_child_mut(W2, f);
        } else {
            f(W2);
        }
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        let (W2, W1) = self;
        if W1.is_ignore() {} else if W1.is_proxy() {
            AnyIdentifiableWidget::foreach_child_rev(W1, f);
        } else {
            f(W1);
        }
        if W2.is_ignore() {} else if W2.is_proxy() {
            AnyIdentifiableWidget::foreach_child_rev(W2, f);
        } else {
            f(W2);
        }
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        let (W1, W2) = self;
        f(W1);
        f(W2);
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        let (W2, W1) = self;
        f(W1);
        f(W2);
    }
}

impl<T: StateContract + PartialEq> AnyIdentifiableWidget<T> for Box<dyn AnyIdentifiableWidget<T>> {
    fn identifier(&self) -> &dyn AnyReadState<T=T> {
        self.deref().identifier()
    }

    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }
}

impl<T: StateContract + PartialEq> WidgetExt for Box<dyn AnyIdentifiableWidget<T>> {

}

impl<T: StateContract + PartialEq> AnyWidget for Box<dyn AnyIdentifiableWidget<T>> {

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

    fn as_widget(&self) -> &dyn AnyWidget {
        self
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