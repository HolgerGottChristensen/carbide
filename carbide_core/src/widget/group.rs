use crate::misc::flags::WidgetFlag;
use crate::widget::{AnyWidget, CommonWidget, Sequence, Widget, WidgetId};
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

pub trait GroupDelegate<T: ?Sized + AnyWidget, I: Sequence<T>, U: ?Sized + AnyWidget, O: Sequence<U>>: Clone + 'static {
    fn call(&self, sequence: I) -> O;
}

impl<K, T: ?Sized + AnyWidget, I: Sequence<T>, U: ?Sized + AnyWidget, O: Sequence<U>> GroupDelegate<T, I, U, O> for K where K: Fn(I) -> O + Clone + 'static {
    fn call(&self, sequence: I) -> O {
        self(sequence)
    }
}

#[derive(Widget)]
pub struct Group<W, T>
where
    T: ?Sized + AnyWidget,
    W: Sequence<T>,
{
    #[id] id: WidgetId,
    sequence: W,
    phantom_data: PhantomData<T>,
}

impl<T: ?Sized + AnyWidget, I: Sequence<T>> Group<I, T> {
    pub fn new(sequence: I) -> Self {
        Group {
            id: WidgetId::new(),
            sequence,
            phantom_data: Default::default(),
        }
    }

    pub fn sequence<U: ?Sized + AnyWidget, O: Sequence<U>, D: GroupDelegate<T, I, U, O>>(sequence: I, delegate: D) -> Group<O, U> {
        let output_sequence = delegate.call(sequence);

        Group {
            id: WidgetId::new(),
            sequence: output_sequence,
            phantom_data: Default::default(),
        }
    }
}

impl<T: ?Sized + AnyWidget, W: Sequence<T>> CommonWidget for Group<W, T> {
    CommonWidgetImpl!(self, flag: WidgetFlag::PROXY);

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        self.sequence.foreach(&mut |child| {
            f(child.as_widget())
        });
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.sequence.foreach_mut(&mut |child| {
            f(child.as_widget_mut())
        });
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.sequence.foreach_rev(&mut |child| {
            f(child.as_widget_mut())
        });
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.sequence.foreach_direct(&mut |child| {
            f(child.as_widget_mut())
        });
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.sequence.foreach_direct_rev(&mut |child| {
            f(child.as_widget_mut())
        });
    }

    fn position(&self) -> Position {
        unreachable!()
    }

    fn set_position(&mut self, _: Position) {
        unreachable!()
    }

    fn dimension(&self) -> Dimension {
        unreachable!()
    }

    fn set_dimension(&mut self, _: Dimension) {
        unreachable!()
    }
}

impl<T: ?Sized + AnyWidget, W: Sequence<T>> Debug for Group<W, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Group")
            .field("sequence", &self.sequence)
            .finish_non_exhaustive()
    }
}

impl<T: ?Sized + AnyWidget, W: Sequence<T>> Clone for Group<W, T> {
    fn clone(&self) -> Self {
        Group {
            id: self.id.clone(),
            sequence: self.sequence.clone(),
            phantom_data: Default::default(),
        }
    }
}