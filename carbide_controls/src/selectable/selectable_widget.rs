use carbide::state::{AnyState, ReadState, State, StateContract};
use std::fmt::Debug;
use std::ops::Deref;
use dyn_clone::DynClone;
use carbide::identifiable::Identifiable;
use carbide::widget::{AnySequence, AnyWidget, Delegate, ForEach, Sequence, WidgetExt, WidgetId, WidgetSync};
use carbide::widget::foreach_widget::ForEachWidget;

pub trait AnySelectableWidget: AnyWidget {
    fn selection(&self) -> &dyn AnyState<T=bool>;

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnySelectableWidget)) {}
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {}
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {}
    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {}
    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {}
}

dyn_clone::clone_trait_object!(AnySelectableWidget);

pub trait SelectableWidget: AnySelectableWidget + WidgetExt + Clone {}

impl<W> SelectableWidget for W where W: AnySelectableWidget + WidgetExt + Clone {}

impl AnySelectableWidget for Box<dyn AnySelectableWidget> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        self.deref().selection()
    }
}

impl WidgetExt for Box<dyn AnySelectableWidget> {

}

impl AnyWidget for Box<dyn AnySelectableWidget> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

impl<
    T: StateContract,
    M: State<T=Vec<T>>,
    W: SelectableWidget,
    U: Delegate<T, W>,
    I: ReadState<T=usize>> AnySelectableWidget for ForEach<T, M, U, W, I> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        todo!()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnySelectableWidget)) {
        (self.children).foreach(f);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.children).foreach_mut(f);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.children).foreach_rev(f);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.children).foreach_direct(f);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.children).foreach_direct_rev(f);
    }
}

impl<
    T: ?Sized + Identifiable<WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: SelectableWidget,
    D: carbide::widget::foreach_widget::Delegate<T, O>
> AnySelectableWidget for ForEachWidget<W, O, D, T> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        todo!()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnySelectableWidget)) {
        (self.content).foreach(f);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.content).foreach_mut(f);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.content).foreach_rev(f);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.content).foreach_direct(f);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        (self.content).foreach_direct_rev(f);
    }
}