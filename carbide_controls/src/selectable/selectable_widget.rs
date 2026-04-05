use carbide::state::{AnyState, ReadState, State, StateContract};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use dyn_clone::DynClone;
use carbide::identifiable::Identifiable;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::widget::{AnySequence, AnyWidget, Delegate, ForEach, Sequence, WidgetExt, WidgetId, WidgetProperties, WidgetSync};
use carbide::widget::foreach_widget::ForEachWidget;
use carbide::widget::properties::WidgetKindDynamic;

pub trait AnySelectableWidget: AnyWidget {
    fn selection(&self) -> &dyn AnyState<T=bool>;

    fn child(&mut self, index: usize) -> &dyn AnySelectableWidget;
    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget));
    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget));
}

dyn_clone::clone_trait_object!(AnySelectableWidget);

pub trait SelectableWidget: AnySelectableWidget + WidgetExt + WidgetProperties + Clone {}

impl<W> SelectableWidget for W where W: AnySelectableWidget + WidgetProperties + WidgetExt + Clone {}

impl AnySelectableWidget for Box<dyn AnySelectableWidget> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        self.deref().selection()
    }

    fn child(&mut self, index: usize) -> &dyn AnySelectableWidget {
        AnySelectableWidget::child(self.deref_mut(), index)
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        AnySelectableWidget::foreach_child(self.deref_mut(), f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        AnySelectableWidget::foreach_child_rev(self.deref_mut(), f)
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
    W: SelectableWidget + AnySequence<dyn AnySelectableWidget>,
    U: Delegate<M, T, W>,
> AnySelectableWidget for ForEach<T, M, U, W> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        unreachable!("When iterating selectable widgets, we should never return proxy widgets, and thus, this should never be called")
    }

    fn child(&mut self, index: usize) -> &dyn AnySelectableWidget {
        self.child::<dyn AnySelectableWidget>(index)
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        self.foreach_child::<dyn AnySelectableWidget>(f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        self.foreach_child_rev::<dyn AnySelectableWidget>(f)
    }
}

impl<
    T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: SelectableWidget + AnySequence<dyn AnySelectableWidget>,
    D: carbide::widget::foreach_widget::Delegate<T, O>
> AnySelectableWidget for ForEachWidget<W, O, D, T> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        unreachable!("When iterating selectable widgets, we should never return proxy widgets, and thus, this should never be called")
    }

    fn child(&mut self, index: usize) -> &dyn AnySelectableWidget {
        self.child::<dyn AnySelectableWidget>(index)
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        self.foreach_child::<dyn AnySelectableWidget>(f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        self.foreach_child_rev::<dyn AnySelectableWidget>(f)
    }
}