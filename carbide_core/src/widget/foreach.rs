use std::fmt::{Debug, Formatter};
use carbide_core::CommonWidgetImpl;


use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{OtherEventHandler, WidgetEvent};
use crate::flags::Flags;
use crate::state::{IndexableState, ReadState, StateContract, TState, ValueState};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

pub trait Delegate<T: StateContract>: Clone {
    fn call(&self, item: TState<T>, index: TState<usize>) -> Box<dyn Widget>;
}

impl<T: StateContract, K> Delegate<T> for K
where
    K: Fn(TState<T>, TState<usize>) -> Box<dyn Widget> + Clone,
{
    fn call(&self, item: TState<T>, index: TState<usize>) -> Box<dyn Widget> {
        self(item, index)
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(OtherEvent)]
pub struct ForEach<T, U>
where
    T: StateContract,
    U: Delegate<T> + 'static,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    #[state]
    model: TState<Vec<T>>,
    delegate: U,

    children: Vec<Box<dyn Widget>>,
    #[state]
    index_offset: TState<usize>,
}

impl<T: StateContract, U: Delegate<T>> ForEach<T, U> {

    #[carbide_default_builder]
    pub fn new(model: impl Into<TState<Vec<T>>>, delegate: U) -> Box<Self> {}

    pub fn new(model: impl Into<TState<Vec<T>>>, delegate: U) -> Box<Self> {
        let model = model.into();

        Box::new(Self {
            id: WidgetId::new(),
            position: Position::default(),
            dimension: Dimension::default(),
            model,
            delegate,
            children: vec![],
            index_offset: ValueState::new(0).into(),
        })
    }

    /*pub fn id_state(mut self, state: Box<dyn State<T, GS>>) -> Box<Self> {
        self.id_state = state;
        Box::new(self)
    }

    pub fn index_state(mut self, state: Box<dyn State<usize, GS>>) -> Box<Self> {
        self.index_state = state;
        Box::new(self)
    }

    pub fn index_offset(mut self, state: Box<dyn State<usize, GS>>) -> Box<Self> {
        self.index_offset = state;
        Box::new(self)
    }*/
}

impl<T: StateContract, U: Delegate<T>> OtherEventHandler for ForEach<T, U> {
    fn handle_other_event(&mut self, _event: &WidgetEvent, _env: &mut Environment) {
        if self.model.value().len() < self.children.len() {
            // Remove the excess elements
            let number_to_remove = self.children.len() - self.model.value().len();
            for _ in 0..number_to_remove {
                self.children.pop();
            }
        } else if self.model.value().len() > self.children.len() {
            // Insert the missing elements
            let number_to_insert = self.model.value().len() - self.children.len();

            for _ in 0..number_to_insert {
                let index = self.children.len();

                let index_state: TState<usize> = ValueState::new(index).into();
                let item_state = self.model.index(&TState::<usize>::from(index));

                let widget = self.delegate.call(item_state.into(), index_state);
                self.children.push(widget);
            }
        }
    }
}

impl<T: StateContract, U: Delegate<T>> CommonWidget for ForEach<T, U> {
    CommonWidgetImpl!(self, id: self.id, children: self.children, position: self.position, dimension: self.dimension, flag: Flags::PROXY);
}

impl<T: StateContract, U: Delegate<T>> Debug for ForEach<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEach")
            .field("children", &self.children)
            .finish()
    }
}

impl<T: StateContract, U: Delegate<T> + 'static> WidgetExt for ForEach<T, U> {}
