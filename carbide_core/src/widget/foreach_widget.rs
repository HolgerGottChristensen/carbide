use crate::draw::{Dimension, Position};
use crate::common::flags::WidgetFlag;
use crate::widget::{CommonWidget, Content, Sequence, Widget, WidgetId, WidgetSync};
use crate::CommonWidgetImpl;
use crate::environment::Environment;
use dyn_clone::DynClone;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
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
#[carbide_exclude(StateSync)]
pub struct ForEachWidget<W, O, D, T>
where
    T: ?Sized + Identifiable<WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: Widget,
    D: Delegate<T, O>
{
    #[id] id: WidgetId,

    sequence: W,
    delegate: D,
    pub content: Content<O>,
    phantom_data: PhantomData<T>,
}

impl<T: ?Sized + Identifiable<WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> Clone for ForEachWidget<W, O, D, T> {
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

impl<T: ?Sized + Identifiable<WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> ForEachWidget<W, O, D, T> {
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


impl<T: ?Sized + Identifiable<WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> WidgetSync for ForEachWidget<W, O, D, T> {
    fn sync(&mut self, env: &mut Environment) {
        // Set the initial index to 0
        let mut index = 0;

        self.sequence.foreach_direct(&mut |child| {
            child.sync(env);
        });

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
                let mut result = self.delegate.call(child);

                // Initialize the widget.
                result.process_initialization(&mut InitializationContext {
                    env: env,
                });

                // Insert the result at the index
                self.content.0.insert_before(index, id, result);
                // Increment the index
                index += 1;
            }
        });

        self.content.1 = index;
    }
}

impl<T: ?Sized + Identifiable<WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> CommonWidget for ForEachWidget<W, O, D, T> {
    CommonWidgetImpl!(self, flag: WidgetFlag::PROXY, child: self.content);

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

impl<T: ?Sized + Identifiable<WidgetId> + WidgetSync + DynClone + 'static, W: Sequence<T>, O: Widget, D: Delegate<T, O>> Debug for ForEachWidget<W, O, D, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEachChild")
            .field("content", &self.content)
            .finish()
    }
}