use std::cell::RefCell;
use carbide::environment::EnvironmentStack;
use carbide::widget::AnyWidget;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use dyn_clone::{clone_box, DynClone};
use indexmap::IndexMap;
use crate::draw::{Dimension, Position};
use crate::flags::WidgetFlag;
use crate::state::StateSync;
use crate::widget::{BuildWidgetIdHasher, CommonWidget, Widget, WidgetExt, WidgetId, Sequence, WidgetSync, Identifiable};
use crate::CommonWidgetImpl;

pub trait Delegate<T: ?Sized, O: Widget>: Clone + 'static {
    fn call(&self, child: Box<T>) -> O;
}

impl<K, O: Widget, T: ?Sized> Delegate<T, O> for K where K: Fn(Box<T>) -> O + Clone + 'static {
    fn call(&self, child: Box<T>) -> O {
        self(child)
    }
}

#[derive(Widget)]
#[carbide_exclude(StateSync)]
pub struct ForEachWidget<W, O, D, T>
where
    T: ?Sized + Identifiable<WidgetId> + DynClone + 'static,
    W: Sequence<T>,
    O: Widget,
    D: Delegate<T, O>
{
    #[id] id: WidgetId,

    sequence: W,
    delegate: D,
    content: (IndexMap<WidgetId, O, BuildWidgetIdHasher>, usize),
    phantom_data: PhantomData<T>,
}

impl<T: ?Sized + Identifiable<WidgetId> + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> Clone for ForEachWidget<W, O, D, T> {
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

impl<T: ?Sized + Identifiable<WidgetId> + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> ForEachWidget<W, O, D, T> {
    pub(crate) fn new(sequence: W, delegate: D) -> Self {
        ForEachWidget {
            id: WidgetId::new(),
            sequence,
            delegate,
            content: (Default::default(), 0),
            phantom_data: Default::default(),
        }
    }
}


impl<T: ?Sized + Identifiable<WidgetId> + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> WidgetSync for ForEachWidget<W, O, D, T> {
    fn sync(&mut self, env: &mut EnvironmentStack) {
        // Set the initial index to 0
        let mut index = 0;

        // For each child of the widget
        self.sequence.foreach(&mut |child| {
            let id = child.id();

            // If the map already contains the key
            if let Some(current) = self.content.0.get_index_of(&id) {

                if current == index {
                    // The content is already at the correct position.
                    index += 1;
                } else if current < index {
                    // If the content exist, but it is currently before the index,
                    // it means two children with the same id exist of the widget.
                    // Skip incrementing the index, and don't move it.
                } else {
                    // Move the content at current, to index
                    self.content.0.move_index(current, index);
                    index += 1;
                }
            } else {
                // Calculate the resulting widget, using the delegate
                let result = self.delegate.call(clone_box(child));
                // Insert the result at the index
                self.content.0.insert_before(index, id, result);
                // Increment the index
                index += 1;
            }
        });

        self.content.1 = index;
    }
}

impl<T: ?Sized + Identifiable<WidgetId> + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> CommonWidget for ForEachWidget<W, O, D, T> {
    CommonWidgetImpl!(self, flag: WidgetFlag::PROXY, child: self.content);

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

impl<T: ?Sized + Identifiable<WidgetId> + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> Debug for ForEachWidget<W, O, D, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEachChild")
            .field("content", &self.content)
            .finish()
    }
}