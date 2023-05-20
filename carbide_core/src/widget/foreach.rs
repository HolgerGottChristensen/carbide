use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use carbide_core::CommonWidgetImpl;


use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{OtherEventHandler, WidgetEvent};
use crate::flags::Flags;
use crate::state::{IndexState, ReadState, RState, State, StateContract, StateExtNew, TState, ValueState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

pub trait Delegate<T: StateContract, O: Widget + Clone>: Clone {
    fn call(&self, item: Box<dyn State<T=T>>, index: Box<dyn State<T=usize>>) -> O;
}

impl<T: StateContract, K, O: Widget + Clone> Delegate<T, O> for K where K: Fn(Box<dyn State<T=T>>, Box<dyn State<T=usize>>) -> O + Clone {
    fn call(&self, item: Box<dyn State<T=T>>, index: Box<dyn State<T=usize>>) -> O {
        self(item, index)
    }
}

#[derive(Clone)]
pub struct EmptyDelegate;

impl Delegate<(), Empty> for EmptyDelegate {
    fn call(&self, _: Box<dyn State<T=()>>, _: Box<dyn State<T=usize>>) -> Empty {
        *Empty::new()
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(OtherEvent)]
pub struct ForEach<T, M, U, W, I>
where
    T: StateContract,
    M: State<T=Vec<T>> + Clone + 'static,
    W: Widget + Clone,
    U: Delegate<T, W> + 'static,
    I: ReadState<T=usize> + Clone + 'static
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    #[state] model: M,
    delegate: U,

    children: Vec<W>,
    #[state] index_offset: I,
    phantom: PhantomData<T>,
}

impl ForEach<(), Vec<()>, EmptyDelegate, Empty, usize> {

    #[carbide_default_builder2]
    pub fn new<T: StateContract, M: State<T=Vec<T>> + Clone, W: Widget + Clone, U: Delegate<T, W>>(model: M, delegate: U) -> Box<ForEach<T, M, U, W, usize>> {
        Box::new(ForEach {
            id: WidgetId::new(),
            position: Position::default(),
            dimension: Dimension::default(),
            model,
            delegate,
            children: vec![],
            index_offset: 0,
            phantom: PhantomData::default()
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

impl<T: StateContract, M: State<T=Vec<T>> + Clone + 'static, W: Widget + Clone, U: Delegate<T, W> + 'static, I: ReadState<T=usize> + Clone + 'static> OtherEventHandler for ForEach<T, M, U, W, I> {
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

                let index_state = ValueState::new(index).as_dyn();

                let mut item_state = IndexState::new(self.model.clone(), index_state.clone());


                let widget = self.delegate.call(item_state.as_dyn(), index_state);
                self.children.push(widget);
            }
        }
    }
}

impl<T: StateContract, M: State<T=Vec<T>> + Clone + 'static, W: Widget + Clone, U: Delegate<T, W> + 'static, I: ReadState<T=usize> + Clone + 'static> CommonWidget for ForEach<T, M, U, W, I> {
    CommonWidgetImpl!(self, id: self.id, children: self.children, position: self.position, dimension: self.dimension, flag: Flags::PROXY);
}

impl<T: StateContract, M: State<T=Vec<T>> + Clone + 'static, W: Widget + Clone, U: Delegate<T, W> + 'static, I: ReadState<T=usize> + Clone + 'static> Debug for ForEach<T, M, U, W, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEach")
            .field("children", &self.children)
            .finish()
    }
}

impl<T: StateContract, M: State<T=Vec<T>> + Clone + 'static, W: Widget + Clone, U: Delegate<T, W> + 'static, I: ReadState<T=usize> + Clone + 'static> WidgetExt for ForEach<T, M, U, W, I> {}
