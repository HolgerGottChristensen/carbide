use carbide::state::{AnyState, ReadState, State, StateContract};
use std::fmt::Debug;
use std::ops::Deref;
use dyn_clone::DynClone;
use carbide::identifiable::Identifiable;
use carbide::widget::{AnySequence, AnyWidget, Delegate, ForEach, RandomAccessCollection, Sequence, WidgetExt, WidgetId, WidgetProperties, WidgetSync};
use carbide::widget::foreach_widget::ForEachWidget;
use carbide::widget::properties::WidgetKindDynamic;

pub trait AnySelectableWidget: AnyWidget {
    fn selection(&self) -> &dyn AnyState<T=bool>;

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn AnySelectableWidget)) {}
    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {}
    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {}
    fn foreach_child_direct(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {}
    fn foreach_child_direct_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {}
}

dyn_clone::clone_trait_object!(AnySelectableWidget);

pub trait SelectableWidget: AnySelectableWidget + WidgetExt + WidgetProperties + Clone {}

impl<W> SelectableWidget for W where W: AnySelectableWidget + WidgetProperties + WidgetExt + Clone {}

impl AnySelectableWidget for Box<dyn AnySelectableWidget> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        self.deref().selection()
    }
}

impl WidgetExt for Box<dyn AnySelectableWidget> {}

impl WidgetProperties for Box<dyn AnySelectableWidget> {
    type Kind = WidgetKindDynamic;
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
    T: StateContract + Identifiable,
    M: RandomAccessCollection<T>,
    W: SelectableWidget,
    U: Delegate<M, T, W>,
> AnySelectableWidget for ForEach<T, M, U, W> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        todo!()
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn AnySelectableWidget)) {
        todo!()//(self.children).foreach(f);
    }

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()//(self.children).foreach_mut(f);
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()//(self.children).foreach_rev(f);
    }

    fn foreach_child_direct(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()//(self.children).foreach_direct(f);
    }

    fn foreach_child_direct_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()//(self.children).foreach_direct_rev(f);
    }
}

impl<
    T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: SelectableWidget,
    D: carbide::widget::foreach_widget::Delegate<T, O>
> AnySelectableWidget for ForEachWidget<W, O, D, T> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        todo!()
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn AnySelectableWidget)) {
        (self.content).foreach(f);
    }

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        (self.content).foreach_mut(f);
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        (self.content).foreach_rev(f);
    }

    fn foreach_child_direct(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        (self.content).foreach_direct(f);
    }

    fn foreach_child_direct_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        (self.content).foreach_direct_rev(f);
    }
}