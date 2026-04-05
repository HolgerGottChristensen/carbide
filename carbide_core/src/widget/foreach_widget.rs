use std::collections::HashMap;
use crate::draw::{Dimension, Position};
use crate::common::flags::WidgetFlag;
use crate::widget::{AnySequence, CommonWidget, Sequence, Widget, WidgetId, WidgetSync};
use crate::CommonWidgetImpl;
use crate::environment::Environment;
use dyn_clone::DynClone;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::state::StateContract;
use carbide::widget::{AnyWidget, ForEach, WidgetProperties};
use carbide::widget::properties::WidgetKindProxy;
use crate::identifiable::Identifiable;
use crate::lifecycle::InitializationContext;

pub trait Delegate<T: ?Sized, O: Widget>: Clone + 'static {
    fn call(&self, child: &T) -> O;
}

impl<K, O: Widget, T: ?Sized> Delegate<T, O> for K where K: Fn(&T) -> O + Clone + 'static {
    fn call(&self, child: &T) -> O {
        self(child)
    }
}

#[derive(Widget)]
#[carbide_exclude(Properties)]
pub struct ForEachWidget<W, O, D, T>
where
    T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: Widget,
    D: Delegate<T, O>
{
    #[id] id: WidgetId,

    sequence: W,
    delegate: D,
    content: HashMap<WidgetId, O, FxBuildHasher>,
    phantom_data: PhantomData<T>,
}

impl<T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> Clone for ForEachWidget<W, O, D, T> {
    fn clone(&self) -> Self {
        ForEachWidget {
            id: WidgetId::new(),
            sequence: self.sequence.clone(),
            delegate: self.delegate.clone(),
            content: self.content.clone(),
            phantom_data: Default::default(),
        }
    }
}

impl<T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> ForEachWidget<W, O, D, T> {
    pub(crate) fn new(sequence: W, delegate: D) -> Self {
        ForEachWidget {
            id: WidgetId::new(),
            sequence,
            delegate,
            content: Default::default(),
            phantom_data: Default::default(),
        }
    }
}

impl<T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> ForEachWidget<W, O, D, T> {
    pub fn child<A: ?Sized>(&mut self, index: usize) -> &mut A where O: AnySequence<A> {
        let inner = self.sequence.index(index);
        let id = inner.id();

        if !self.content.contains_key(&id) {
            self.content.insert(id, self.delegate.call(inner));
        }

        let widget = self.content.get_mut(&id)
            .expect("The widget with the id to be inserted in the statement above");

        // This is some type magic because i cant constrain O to implement A
        <O as AnySequence<A>>::index(widget, 0)
    }

    pub fn foreach_child<A: ?Sized>(&mut self, f: &mut dyn FnMut(&mut A)) where O: AnySequence<A> {
        self.sequence.foreach(&mut |inner_child| {
            let id = inner_child.id();

            if !self.content.contains_key(&id) {
                self.content.insert(id, self.delegate.call(inner_child));
            }

            let widget = self.content.get_mut(&id).expect("The widget with the id to be inserted in the statement above");

            f(<O as AnySequence<A>>::index(widget, 0))
        })
    }

    pub fn foreach_child_rev<A: ?Sized>(&mut self, f: &mut dyn FnMut(&mut A)) where O: AnySequence<A> {
        self.sequence.foreach_rev(&mut |inner_child| {
            let id = inner_child.id();

            if !self.content.contains_key(&id) {
                self.content.insert(id, self.delegate.call(inner_child));
            }

            let widget = self.content.get_mut(&id).expect("The widget with the id to be inserted in the statement above");

            f(<O as AnySequence<A>>::index(widget, 0))
        })
    }
}

impl<T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> CommonWidget for ForEachWidget<W, O, D, T> {
    CommonWidgetImpl!(self, flag: WidgetFlag::PROXY);

    fn child(&mut self, index: usize) -> &mut dyn AnyWidget {
        self.child::<dyn AnyWidget>(index)
    }

    fn child_count(&mut self) -> usize {
        self.sequence.count()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        self.foreach_child::<dyn AnyWidget>(f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        self.foreach_child_rev::<dyn AnyWidget>(f)
    }

    fn position(&self) -> Position {
        unimplemented!()
    }

    fn set_position(&mut self, _: Position) {
        unimplemented!()
    }

    fn dimension(&self) -> Dimension {
        unimplemented!()
    }

    fn set_dimension(&mut self, _: Dimension) {
        unimplemented!()
    }
}

impl<T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> WidgetProperties for  ForEachWidget<W, O, D, T>
{
    type Kind = WidgetKindProxy;
}

impl<T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> Debug for ForEachWidget<W, O, D, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEachChild")
            .field("content", &self.content)
            .finish()
    }
}