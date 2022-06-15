use std::fmt::{Debug, Formatter};

use crate::draw::{Dimension, Position};
use crate::event::{OtherEventHandler, WidgetEvent};
use crate::prelude::*;

pub trait Delegate<T: StateContract>: Clone {
    fn call(&self, item: TState<T>, index: UsizeState) -> Box<dyn Widget>;
}

impl<T: StateContract, K> Delegate<T> for K where K: Fn(TState<T>, UsizeState) -> Box<dyn Widget> + Clone {
    fn call(&self, item: TState<T>, index: UsizeState) -> Box<dyn Widget> {
        self(item, index)
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(OtherEvent)]
pub struct ForEach<T, U> where T: StateContract, U: Delegate<T> {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    #[state] model: TState<Vec<T>>,
    delegate: U,

    children: Vec<Box<dyn Widget>>,
    #[state] index_offset: UsizeState,
}

impl<T: StateContract, U: Delegate<T>> ForEach<T, U> {
    pub fn new<K: Into<TState<Vec<T>>>>(model: K, delegate: U) -> Box<Self> {
        let model = model.into();
        let mut list: Vec<Box<dyn Widget>> = vec![];

        for (index, _element) in model.value().deref().iter().enumerate() {
            let index_state: UsizeState = ValueState::new(index).into();
            let item_state = VecState::new(model.clone(), index);

            let widget = delegate.call(item_state.into(), index_state);
            list.push(widget);
        }

        Box::new(Self {
            id: WidgetId::new(),
            position: Position::default(),
            dimension: Dimension::default(),
            model,
            delegate,
            children: list,
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
        if self.model.value().len() < self.children.len() { // Remove the excess elements
            let number_to_remove = self.children.len() - self.model.value().len();
            for _ in 0..number_to_remove {
                self.children.pop();
            }
        } else if self.model.value().len() > self.children.len() { // Insert the missing elements
            let number_to_insert = self.model.value().len() - self.children.len();

            for _ in 0..number_to_insert {
                let index = self.children.len();

                let index_state: UsizeState = ValueState::new(index).into();
                let item_state = VecState::new(self.model.clone(), index);


                let widget = self.delegate.call(item_state.into(), index_state);
                self.children.push(widget);
            }
        }
    }
}

impl<T: StateContract, U: Delegate<T>> CommonWidget for ForEach<T, U> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn flag(&self) -> Flags {
        Flags::PROXY
    }

    fn children(&self) -> WidgetIter {
        let contains_proxy_or_ignored = self.children.iter().fold(false, |a, b| a || (b.flag() == Flags::PROXY || b.flag() == Flags::IGNORE));
        if !contains_proxy_or_ignored {
            WidgetIter::Vec(self.children.iter())
        } else {
            self.children
                .iter()
                .filter(|x| x.flag() != Flags::IGNORE)
                .rfold(WidgetIter::Empty, |acc, x| {
                    if x.flag() == Flags::PROXY {
                        WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
                    } else {
                        WidgetIter::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        let contains_proxy_or_ignored = self.children.iter().fold(false, |a, b| a || (b.flag() == Flags::PROXY || b.flag() == Flags::IGNORE));
        if !contains_proxy_or_ignored {
            WidgetIterMut::Vec(self.children.iter_mut())
        } else {
            self.children
                .iter_mut()
                .filter(|x| x.flag() != Flags::IGNORE)
                .rfold(WidgetIterMut::Empty, |acc, x| {
                    if x.flag() == Flags::PROXY {
                        WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                    } else {
                        WidgetIterMut::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Vec(self.children.iter_mut())
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::VecRev(self.children.iter_mut().rev())
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

impl<T: StateContract, U: Delegate<T>> Debug for ForEach<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEach")
            .field("children", &self.children)
            .finish()
    }
}

impl<T: StateContract, U: Delegate<T> + 'static> WidgetExt for ForEach<T, U> {}
