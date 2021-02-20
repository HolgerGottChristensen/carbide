#![allow(unsafe_code)]

use std::collections::HashMap;
use std::fmt::Debug;

use crate::prelude::*;
use crate::state::state::CommonState;
use crate::widget::render::ChildRender;
use std::hash::Hash;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait ForEachDelegate: Clone + PartialEq + Eq + Hash + Debug + Serialize + DeserializeOwned + Default {}

impl<T> ForEachDelegate for T where T: Clone + PartialEq + Eq + Hash + Debug + Serialize + DeserializeOwned + Default {}

#[derive(Debug, Clone, Widget)]
#[state_sync(sync_state)]
pub struct ForEach<GS, T> where GS: GlobalState, T: ForEachDelegate + 'static {
    id: Uuid, // --
    children_map: HashMap<T, Box<dyn Widget<GS>>>,
    delegate: Box<dyn Widget<GS>>,
    #[state] ids: Box<dyn State<Vec<T>, GS>>,
    position: Point, // --
    dimension: Dimensions, // --
    id_state: Box<dyn State<T, GS>>,
    index_state: Box<dyn State<usize, GS>>,
    #[state] index_offset: Box<dyn State<usize, GS>>,
}

impl<GS: GlobalState, T: ForEachDelegate + 'static> WidgetExt<GS> for ForEach<GS, T> {}

impl<GS: GlobalState, T: ForEachDelegate + 'static> ForEach<GS, T> {
    pub fn new(ids: Box<dyn State<Vec<T>, GS>>, delegate: Box<dyn Widget<GS>>) -> Box<Self> {

        let mut map = HashMap::new();

        for i in ids.get_latest_value() {
            map.insert(i.clone(), Clone::clone(&delegate));
        }

        Box::new(Self {
            id: Uuid::new_v4(),
            children_map: map,
            delegate,
            ids,
            position: [100.0,100.0],
            dimension: [100.0,100.0],
            id_state: Box::new(CommonState::new_local_with_key(&T::default())),
            index_state: Box::new(CommonState::new_local_with_key(&0)),
            index_offset: Box::new(CommonState::new_local_with_key(&0)),
        })
    }

    pub fn id_state(mut self, state: Box<dyn State<T, GS>>) -> Box<Self> {
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
    }

    fn sync_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
        self.update_all_widget_state(env, global_state);

        self.insert_local_state(env);

        let mut ids = self.ids.clone();

        let initial_offset = *self.index_offset.get_latest_value();
        let id_key = self.id_state.get_key().unwrap().clone();
        let index_key = self.index_state.get_key().unwrap().clone();

        for (i, child) in self.get_proxied_children().enumerate() {

            env.insert_local_state_from_key_value(&id_key, &ids.get_value(global_state)[i]);
            env.insert_local_state_from_key_value(&index_key, &(i + initial_offset));

            child.sync_state(env, global_state)
        }

        self.update_local_widget_state(env);
    }
}

impl<GS: GlobalState, T: ForEachDelegate> CommonWidget<GS> for ForEach<GS, T> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::PROXY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        let mut w = WidgetIter::Empty;

        for id in self.ids.get_latest_value().iter().rev() {
            let item = self.children_map.get(id).unwrap();

            if item.get_flag() == Flags::PROXY {
                w = WidgetIter::Multi(Box::new(item.get_children()), Box::new(w));
            } else {
                w = WidgetIter::Single(item, Box::new(w))
            }
        }

        w
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        let mut w = WidgetIterMut::Empty;

        for id in self.ids.get_latest_value().iter().rev() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains{
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }

        for id in self.ids.get_latest_value().iter().rev() {

            let item: &mut Box<dyn Widget<GS>> = unsafe {
                let p: *mut Box<dyn Widget<GS>> = self.children_map.get_mut(id).unwrap();
                p.as_mut().unwrap()
            };

            if item.get_flag() == Flags::PROXY {
                w = WidgetIterMut::Multi(Box::new(item.get_children_mut()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        let mut w = WidgetIterMut::Empty;

        for id in self.ids.get_latest_value().iter().rev() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains{
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }

        for id in self.ids.get_latest_value().iter().rev() {
            let item: &mut Box<dyn Widget<GS>> = unsafe {
                let p: *mut Box<dyn Widget<GS>> = self.children_map.get_mut(id).unwrap();
                p.as_mut().unwrap()
            };

            if item.get_flag() == Flags::PROXY {
                w = WidgetIterMut::Multi(Box::new(item.get_proxied_children()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        let mut w = WidgetIterMut::Empty;

        for id in self.ids.get_latest_value().iter() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains{
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }

        for id in self.ids.get_latest_value().iter() {
            let item: &mut Box<dyn Widget<GS>> = unsafe {
                let p: *mut Box<dyn Widget<GS>> = self.children_map.get_mut(id).unwrap();
                p.as_mut().unwrap()
            };

            if item.get_flag() == Flags::PROXY {
                w = WidgetIterMut::Multi(Box::new(item.get_proxied_children()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S: GlobalState, T: ForEachDelegate> ChildRender for ForEach<S, T> {}

impl<S: GlobalState, T: ForEachDelegate> Layout<S> for ForEach<S, T> {
    fn flexibility(&self) -> u32 {
        unimplemented!()
    }

    fn calculate_size(&mut self, _requested_size: Dimensions, _env: &Environment<S>) -> Dimensions {
        unimplemented!()
    }

    fn position_children(&mut self) {
        unimplemented!()
    }
}
