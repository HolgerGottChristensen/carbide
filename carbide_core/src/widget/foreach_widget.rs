use std::cell::RefCell;
use carbide::environment::EnvironmentStack;
use carbide::widget::AnyWidget;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use dyn_clone::clone_box;
use indexmap::IndexMap;
use crate::draw::{Dimension, Position};
use crate::flags::WidgetFlag;
use crate::state::StateSync;
use crate::widget::{BuildWidgetIdHasher, CommonWidget, Widget, WidgetExt, WidgetId, WidgetSequence, WidgetSync};
use crate::CommonWidgetImpl;

pub trait Delegate<O: Widget>: Clone + 'static {
    fn call(&self, child: Box<dyn AnyWidget>) -> O;
}

impl<K, O: Widget> Delegate<O> for K where K: Fn(Box<dyn AnyWidget>) -> O + Clone + 'static {
    fn call(&self, child: Box<dyn AnyWidget>) -> O {
        self(child)
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct ForEachWidget<W, O, D>
where
    W: WidgetSequence,
    O: Widget,
    D: Delegate<O>
{
    id: WidgetId,

    sequence: W,
    delegate: D,
    content: (IndexMap<WidgetId, O, BuildWidgetIdHasher>, usize),
}

impl<W: WidgetSequence, O: Widget, D: Delegate<O>> ForEachWidget<W, O, D> {
    pub(crate) fn new(sequence: W, delegate: D) -> Self {
        ForEachWidget {
            id: WidgetId::new(),
            sequence,
            delegate,
            content: (Default::default(), 0),
        }
    }
}


impl<W: WidgetSequence, O: Widget, D: Delegate<O>> WidgetSync for ForEachWidget<W, O, D> {
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

impl<W: WidgetSequence, O: Widget, D: Delegate<O>> CommonWidget for ForEachWidget<W, O, D> {
    CommonWidgetImpl!(self, id: self.id, flag: WidgetFlag::PROXY, child: self.content);

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

impl<W: WidgetSequence, O: Widget, D: Delegate<O>> Debug for ForEachWidget<W, O, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEachChild")
            .field("content", &self.content)
            .finish()
    }
}