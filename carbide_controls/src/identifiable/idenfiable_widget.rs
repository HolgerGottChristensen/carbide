use carbide::state::StateContract;
use carbide::widget::{AnyWidget, Identifiable, WidgetExt};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub trait AnyIdentifiableWidget<T>: AnyWidget + Identifiable<T> where T: StateContract + PartialEq {
    fn as_widget(&self) -> &dyn AnyWidget;

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<T>));
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>));
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>));
    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>));
    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>));
}

impl<T: StateContract + PartialEq, W> AnyIdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + Clone {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<T>)) {
        todo!()
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        todo!()
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        todo!()
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        todo!()
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        todo!()
    }
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T>);

pub trait IdentifiableWidget<T>: AnyIdentifiableWidget<T> + WidgetExt + Clone where T: StateContract + PartialEq  {}

impl<T: StateContract + PartialEq, W> IdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + WidgetExt + Clone {}