#![allow(unsafe_code)]

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use fxhash::{FxBuildHasher, FxHashMap};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::draw::{Dimension, Position};
use crate::event::{KeyboardEvent, MouseEvent};
use crate::prelude::*;

pub trait Delegate<T: StateContract + Identifiable>: Fn(TState<T>, UsizeState) -> Box<dyn Widget> + Clone {}

impl<I, T: StateContract + Identifiable> Delegate<T> for I where I: Fn(TState<T>, UsizeState) -> Box<dyn Widget> + Clone {}

pub trait Identifiable {
    fn id(&self) -> Uuid;
}

#[derive(Clone, Widget)]
pub struct ForEach<T, U> where T: StateContract + Identifiable, U: Delegate<T> {
    id: Uuid,
    position: Position,
    dimension: Dimension,

    #[state] model: TState<Vec<T>>,
    delegate: U,

    children: FxHashMap<Uuid, Box<dyn Widget>>,
    #[state] index_offset: UsizeState,
}

impl<T: StateContract + Identifiable + 'static, U: Delegate<T>> ForEach<T, U> {
    pub fn new<K: Into<TState<Vec<T>>>>(model: K, delegate: U) -> Box<Self> {
        let model = model.into();
        let mut map = HashMap::with_hasher(FxBuildHasher::default());

        for (index, element) in model.value().deref().iter().enumerate() {
            let id = element.id();
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
            map.insert(id, widget);
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

impl<T: StateContract + Identifiable, U: Delegate<T>> CommonWidget for ForEach<T, U> {
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
        let mut w = WidgetIter::Empty;

        for item in self.model.value().iter().rev() {
            let id = item.id();
            let item = self.children.get(&id).unwrap();

            if item.flag() == Flags::PROXY {
                w = WidgetIter::Multi(Box::new(item.children()), Box::new(w));
            } else {
                w = WidgetIter::Single(item, Box::new(w))
            }
        }

        w
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        let mut w = WidgetIterMut::Empty;

        /*for id in self.model.value().iter().rev() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains {
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }*/

        for item in self.model.value().iter().rev() {
            let id = item.id();
            let item: &mut Box<dyn Widget> = unsafe {
                let p: *mut Box<dyn Widget> = self.children.get_mut(&id).unwrap();
                p.as_mut().unwrap()
            };

            if item.flag() == Flags::PROXY {
                w = WidgetIterMut::Multi(Box::new(item.children_mut()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        let mut w = WidgetIterMut::Empty;

        /*for id in self.ids.iter().rev() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains {
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }*/

        for item in self.model.value().iter().rev() {
            let id = item.id();
            let item: &mut Box<dyn Widget> = unsafe {
                let p: *mut Box<dyn Widget> = self.children.get_mut(&id).unwrap();
                p.as_mut().unwrap()
            };

            if item.flag() == Flags::PROXY {
                w = WidgetIterMut::Multi(Box::new(item.proxied_children()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        let mut w = WidgetIterMut::Empty;

        /*for id in self.ids.iter() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains {
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }*/

        for item in self.model.value().iter().rev() {
            let id = item.id();
            let item: &mut Box<dyn Widget> = unsafe {
                let p: *mut Box<dyn Widget> = self.children.get_mut(&id).unwrap();
                p.as_mut().unwrap()
            };

            if item.flag() == Flags::PROXY {
                w = WidgetIterMut::Multi(Box::new(item.proxied_children_rev()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
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

impl<T: StateContract + Identifiable + 'static, U: Delegate<T> + 'static> WidgetExt for ForEach<T, U> {}
