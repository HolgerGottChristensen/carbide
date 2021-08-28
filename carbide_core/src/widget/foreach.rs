use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use fxhash::{FxBuildHasher, FxHashMap};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::draw::{Dimension, Position};
use crate::event::{KeyboardEvent, MouseEvent};
use crate::prelude::*;

pub trait Delegate<T: StateContract, W: Widget>: Fn(TState<T>, UsizeState) -> W + Clone {}

impl<I, T: StateContract, W: Widget> Delegate<T, W> for I where I: Fn(TState<T>, UsizeState) -> W + Clone {}

#[derive(Clone, Widget)]
pub struct ForEach<T, U, W> where T: StateContract, W: Widget + Clone, U: Delegate<T, W> {
    id: Uuid,
    position: Position,
    dimension: Dimension,

    #[state] model: TState<Vec<T>>,
    delegate: U,

    children: Vec<W>,
    #[state] index_offset: UsizeState,
}

impl<T: StateContract + 'static, W: Widget + Clone, U: Delegate<T, W>> ForEach<T, U, W> {
    pub fn new<K: Into<TState<Vec<T>>>>(model: K, delegate: U) -> Box<Self> {
        let model = model.into();
        let mut map = vec![];

        for (index, element) in model.value().deref().iter().enumerate() {
            let index_state: UsizeState = ValueState::new(index).into();
            let item_state: MapState<Vec<T>, T, usize> = MapState::new(model.clone(),
                                                                       index,
                                                                       |a, index| {
                                                                           &a[index]
                                                                       },
                                                                       |a, index| {
                                                                           &mut a[index]
                                                                       });
            let widget = (delegate)(item_state.into(), index_state);
            map.push(widget);
        }

        Box::new(Self {
            id: Uuid::new_v4(),
            position: Position::default(),
            dimension: Dimension::default(),
            model,
            delegate,
            children: map,
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

impl<T: StateContract, W: Widget + Clone, U: Delegate<T, W>> CommonWidget for ForEach<T, U, W> {
    fn id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::PROXY
    }

    fn children(&self) -> WidgetIter {
        self.children
            .iter()
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.flag() == Flags::PROXY {
                    WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.flag() == Flags::PROXY {
                    WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .fold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.dimension = dimensions
    }
}

impl<T: StateContract + 'static, W: Widget + Clone + 'static, U: Delegate<T, W> + 'static> WidgetExt for ForEach<T, U, W> {}
