use std::fmt::{Debug, Formatter};
use dyn_clone::clone_box;
use indexmap::IndexMap;
use carbide::CommonWidgetImpl;
use carbide::draw::{Dimension, Position};
use carbide::environment::EnvironmentStack;
use carbide::flags::WidgetFlag;
use carbide::state::{AnyState, State};
use carbide::widget::{AnyWidget, BuildWidgetIdHasher, CommonWidget, Widget, WidgetId, WidgetSync};
/*use crate::identifiable::SelectableSequence;

pub trait Delegate<I: Widget, S: State<T=bool>, O: Widget>: Clone + 'static {
    fn call(&self, item: I, selected: S) -> O;
}

impl<I: Widget, S: State<T=bool>, O: Widget, K> Delegate<I, S, O> for K where K: Fn(I, S) -> O + Clone + 'static {
    fn call(&self, item: I, selected: S) -> O {
        self(item, selected)
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct SelectableForEach<S, D, O>
where
    D: Delegate<Box<dyn AnyWidget>, Box<dyn AnyState<T=bool>>, O>,
    O: Widget,
    S: SelectableSequence + Clone,
{
    #[id] id: WidgetId,
    delegate: D,
    sequence: S,
    children: IndexMap<WidgetId, O, BuildWidgetIdHasher>,
}

impl<D: Delegate<Box<dyn AnyWidget>, Box<dyn AnyState<T=bool>>, O>, O: Widget, S: SelectableSequence + Clone> SelectableForEach<S, D, O> {
    pub fn new(sequence: S, delegate: D) -> SelectableForEach<S, D, O> {
        SelectableForEach {
            id: WidgetId::new(),
            delegate,
            sequence,
            children: IndexMap::default(),
        }
    }
}

impl<D: Delegate<Box<dyn AnyWidget>, Box<dyn AnyState<T=bool>>, O>, O: Widget, S: SelectableSequence + Clone> WidgetSync for SelectableForEach<S, D, O> {
    fn sync(&mut self, env: &mut EnvironmentStack) {
        if self.sequence.has_changed(&mut self.children.keys().copied()) {
            self.children.clear();

            self.sequence.update(&mut |widget, selected| {
                self.children.insert(
                    widget.id(),
                    self.delegate.call(clone_box(widget), selected)
                );
            });
        }
    }
}

impl<D: Delegate<Box<dyn AnyWidget>, Box<dyn AnyState<T=bool>>, O>, O: Widget, S: SelectableSequence + Clone> CommonWidget for SelectableForEach<S, D, O> {
    CommonWidgetImpl!(self, flag: WidgetFlag::PROXY);

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        for (_, child) in self.children.iter() {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut() {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child_mut(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut().rev() {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child_rev(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut() {
            f(child);
        }
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut().rev() {
            f(child);
        }
    }

    fn position(&self) -> Position {
        unimplemented!()
    }

    fn set_position(&mut self, position: Position) {
        unimplemented!()
    }

    fn dimension(&self) -> Dimension {
        unimplemented!()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        unimplemented!()
    }
}

impl<D: Delegate<Box<dyn AnyWidget>, Box<dyn AnyState<T=bool>>, O>, O: Widget, S: SelectableSequence + Clone> Debug for SelectableForEach<S, D, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}*/