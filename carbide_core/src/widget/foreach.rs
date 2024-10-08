use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::WidgetFlag;
use crate::state::{AnyReadState, AnyState, IgnoreWritesState, IndexState, IntoReadState, IntoState, StateSync, ReadState, ReadStateExtNew, State, StateContract, StateExtNew, ValueState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetSync};

pub trait Delegate<T: StateContract, O: Widget>: Clone + 'static {
    fn call(&self, item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyReadState<T=usize>>) -> O;
}

impl<T: StateContract, K, O: Widget> Delegate<T, O> for K where K: Fn(Box<dyn AnyState<T=T>>, Box<dyn AnyReadState<T=usize>>) -> O + Clone + 'static {
    fn call(&self, item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyReadState<T=usize>>) -> O {
        self(item, index)
    }
}

#[derive(Clone)]
pub struct EmptyDelegate;

impl Delegate<(), Empty> for EmptyDelegate {
    fn call(&self, _: Box<dyn AnyState<T=()>>, _: Box<dyn AnyReadState<T=usize>>) -> Empty {
        Empty::new()
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct ForEach<T, M, U, W, I>
where
    T: StateContract,
    M: State<T=Vec<T>>,
    W: Widget,
    U: Delegate<T, W>,
    I: ReadState<T=usize>
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
    pub fn new<T: StateContract, M: IntoState<Vec<T>>, W: Widget, U: Delegate<T, W>>(model: M, delegate: U) -> ForEach<T, M::Output, U, W, usize> {
        ForEach {
            id: WidgetId::new(),
            position: Position::default(),
            dimension: Dimension::default(),
            model: model.into_state(),
            delegate,
            children: vec![],
            index_offset: 0,
            phantom: PhantomData::default()
        }
    }

    pub fn new_read<T: StateContract, M: IntoReadState<Vec<T>>, W: Widget, U: Delegate<T, W>>(model: M, delegate: U) -> ForEach<T, IgnoreWritesState<Vec<T>, M::Output>, U, W, usize> {
        ForEach {
            id: WidgetId::new(),
            position: Position::default(),
            dimension: Dimension::default(),
            model: model.into_read_state().ignore_writes(),
            delegate,
            children: vec![],
            index_offset: 0,
            phantom: PhantomData::default()
        }
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

impl<T, M, U, W, I> WidgetSync for ForEach<T, M, U, W, I>
    where
        T: StateContract,
        M: State<T=Vec<T>>,
        W: Widget,
        U: Delegate<T, W>,
        I: ReadState<T=usize>
{
    fn sync(&mut self, env: &mut Environment) {
        self.model.sync(env);
        self.index_offset.sync(env);

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

                let index_state = ValueState::new(index).as_dyn_read();

                let item_state = IndexState::new(self.model.clone(), index);


                let widget = self.delegate.call(item_state.as_dyn(), index_state);
                self.children.push(widget);
            }
        }
    }
}

impl<T: StateContract, M: State<T=Vec<T>>, W: Widget, U: Delegate<T, W>, I: ReadState<T=usize>> CommonWidget for ForEach<T, M, U, W, I> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension, flag: WidgetFlag::PROXY);
}

impl<T: StateContract, M: State<T=Vec<T>>, W: Widget, U: Delegate<T, W>, I: ReadState<T=usize>> Debug for ForEach<T, M, U, W, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEach")
            .field("children", &self.children)
            .finish()
    }
}