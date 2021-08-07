// #![allow(unsafe_code)]
//
// use std::collections::HashMap;
// use std::fmt::Debug;
// use std::hash::Hash;
//
// use fxhash::{FxBuildHasher, FxHashMap};
// use serde::de::DeserializeOwned;
// use serde::Serialize;
//
// use crate::event_handler::{KeyboardEvent, MouseEvent};
// use crate::prelude::*;
// use crate::state::local_state::LocalState;
// use crate::widget::render::ChildRender;
//
// pub trait ForEachDelegate: Clone + PartialEq + Eq + Hash + Debug + Serialize + DeserializeOwned + Default {}
//
// impl<T> ForEachDelegate for T where T: Clone + PartialEq + Eq + Hash + Debug + Serialize + DeserializeOwned + Default {}
//
// #[derive(Debug, Clone, Widget)]
// //#[state_sync(sync_state)]
// //#[event(process_mouse_event, process_keyboard_event)]
// pub struct ForEach<GS, T> where GS: GlobalStateContract, T: ForEachDelegate + 'static {
//     id: Uuid,
//     // --
//     children_map: FxHashMap<T, Box<dyn Widget<GS>>>,
//     delegate: Box<dyn Widget<GS>>,
//     #[state] ids: TState<Vec<T>, GS>,
//     position: Point,
//     // --
//     dimension: Dimensions,
//     // --
//     id_state: Box<dyn State<T, GS>>,
//     index_state: Box<dyn State<usize, GS>>,
//     #[state] index_offset: Box<dyn State<usize, GS>>,
// }
//
// impl<GS: GlobalStateContract, T: ForEachDelegate + 'static> WidgetExt<GS> for ForEach<GS, T> {}
//
// impl<GS: GlobalStateContract, T: ForEachDelegate + 'static> ForEach<GS, T> {
//     pub fn new<K: Into<TState<Vec<T>, GS>>>(ids: K, delegate: Box<dyn Widget<GS>>) -> Box<Self> {
//         let ids = ids.into();
//         let mut map = HashMap::with_hasher(FxBuildHasher::default());
//
//         for i in ids.deref() {
//             map.insert(i.clone(), Clone::clone(&delegate));
//         }
//
//         Box::new(Self {
//             id: Uuid::new_v4(),
//             children_map: map,
//             delegate,
//             ids,
//             position: [100.0, 100.0],
//             dimension: [100.0, 100.0],
//             id_state: Box::new(LocalState::new(T::default())),
//             index_state: Box::new(LocalState::new(0)),
//             index_offset: Box::new(LocalState::new(0)),
//         })
//     }
//
//     pub fn id_state(mut self, state: Box<dyn State<T, GS>>) -> Box<Self> {
//         self.id_state = state;
//         Box::new(self)
//     }
//
//     pub fn index_state(mut self, state: Box<dyn State<usize, GS>>) -> Box<Self> {
//         self.index_state = state;
//         Box::new(self)
//     }
//
//     pub fn index_offset(mut self, state: Box<dyn State<usize, GS>>) -> Box<Self> {
//         self.index_offset = state;
//         Box::new(self)
//     }
//
//     /*fn sync_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
//         self.update_all_widget_state(env, global_state);
//
//         self.insert_local_state(env);
//
//         let mut ids = self.ids.clone();
//
//         let initial_offset = *self.index_offset.get_latest_value();
//         let id_key = self.id_state.get_key().unwrap().clone();
//         let index_key = self.index_state.get_key().unwrap().clone();
//
//         for (i, child) in self.get_proxied_children().enumerate() {
//             env.insert_local_state_from_key_value(&id_key, &ids.get_value(env, global_state)[i]);
//             env.insert_local_state_from_key_value(&index_key, &(i + initial_offset));
//
//             child.sync_state(env, global_state)
//         }
//
//         self.update_local_widget_state(env);
//     }
//
//     fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &mut GS) {
//         self.update_all_widget_state(env, global_state);
//
//         self.insert_local_state(env);
//
//         let mut ids = self.ids.clone();
//
//         let initial_offset = *self.index_offset.get_latest_value();
//         let id_key = self.id_state.get_key().unwrap().clone();
//         let index_key = self.index_state.get_key().unwrap().clone();
//
//         for (i, child) in self.get_proxied_children().enumerate() {
//             env.insert_local_state_from_key_value(&id_key, &ids.get_value(env, global_state)[i]);
//             env.insert_local_state_from_key_value(&index_key, &(i + initial_offset));
//
//             child.process_mouse_event(event, &consumed, env, global_state);
//             if *consumed { return () }
//         }
//
//         self.update_local_widget_state(env)
//     }
//
//     fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
//         self.update_all_widget_state(env, global_state);
//
//         self.insert_local_state(env);
//
//         let mut ids = self.ids.clone();
//
//         let initial_offset = *self.index_offset.get_latest_value();
//         let id_key = self.id_state.get_key().unwrap().clone();
//         let index_key = self.index_state.get_key().unwrap().clone();
//
//         for (i, child) in self.get_proxied_children().enumerate() {
//             env.insert_local_state_from_key_value(&id_key, &ids.get_value(env, global_state)[i]);
//             env.insert_local_state_from_key_value(&index_key, &(i + initial_offset));
//
//             child.process_keyboard_event(event, env, global_state);
//         }
//
//         self.update_local_widget_state(env)
//     }*/
// }
//
// impl<GS: GlobalStateContract, T: ForEachDelegate> CommonWidget<GS> for ForEach<GS, T> {
//     fn get_id(&self) -> Uuid {
//         self.id
//     }
//
//     fn set_id(&mut self, id: Uuid) {
//         self.id = id;
//     }
//
//     fn get_flag(&self) -> Flags {
//         Flags::PROXY
//     }
//
//     fn get_children(&self) -> WidgetIter<GS> {
//         let mut w = WidgetIter::Empty;
//
//         for id in self.ids.iter().rev() {
//             let item = self.children_map.get(id).unwrap();
//
//             if item.get_flag() == Flags::PROXY {
//                 w = WidgetIter::Multi(Box::new(item.get_children()), Box::new(w));
//             } else {
//                 w = WidgetIter::Single(item, Box::new(w))
//             }
//         }
//
//         w
//     }
//
//     fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
//         let mut w = WidgetIterMut::Empty;
//
//         for id in self.ids.iter().rev() {
//             let contains = self.children_map.contains_key(id).clone();
//             if !contains {
//                 self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
//             }
//         }
//
//         for id in self.ids.iter().rev() {
//             let item: &mut Box<dyn Widget<GS>> = unsafe {
//                 let p: *mut Box<dyn Widget<GS>> = self.children_map.get_mut(id).unwrap();
//                 p.as_mut().unwrap()
//             };
//
//             if item.get_flag() == Flags::PROXY {
//                 w = WidgetIterMut::Multi(Box::new(item.get_children_mut()), Box::new(w));
//             } else {
//                 w = WidgetIterMut::Single(item, Box::new(w))
//             }
//         }
//
//         w
//     }
//
//     fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
//         let mut w = WidgetIterMut::Empty;
//
//         for id in self.ids.iter().rev() {
//             let contains = self.children_map.contains_key(id).clone();
//             if !contains {
//                 self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
//             }
//         }
//
//         for id in self.ids.iter().rev() {
//             let item: &mut Box<dyn Widget<GS>> = unsafe {
//                 let p: *mut Box<dyn Widget<GS>> = self.children_map.get_mut(id).unwrap();
//                 p.as_mut().unwrap()
//             };
//
//             if item.get_flag() == Flags::PROXY {
//                 w = WidgetIterMut::Multi(Box::new(item.get_proxied_children()), Box::new(w));
//             } else {
//                 w = WidgetIterMut::Single(item, Box::new(w))
//             }
//         }
//
//         w
//     }
//
//     fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
//         let mut w = WidgetIterMut::Empty;
//
//         for id in self.ids.iter() {
//             let contains = self.children_map.contains_key(id).clone();
//             if !contains {
//                 self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
//             }
//         }
//
//         for id in self.ids.iter() {
//             let item: &mut Box<dyn Widget<GS>> = unsafe {
//                 let p: *mut Box<dyn Widget<GS>> = self.children_map.get_mut(id).unwrap();
//                 p.as_mut().unwrap()
//             };
//
//             if item.get_flag() == Flags::PROXY {
//                 w = WidgetIterMut::Multi(Box::new(item.get_proxied_children()), Box::new(w));
//             } else {
//                 w = WidgetIterMut::Single(item, Box::new(w))
//             }
//         }
//
//         w
//     }
//
//     fn get_position(&self) -> Point {
//         self.position
//     }
//
//     fn set_position(&mut self, position: Dimensions) {
//         self.position = position;
//     }
//
//     fn get_dimension(&self) -> Dimensions {
//         self.dimension
//     }
//
//     fn set_dimension(&mut self, dimensions: Dimensions) {
//         self.dimension = dimensions
//     }
// }
//
// impl<GS: GlobalStateContract, T: ForEachDelegate> ChildRender for ForEach<GS, T> {}
//
// impl<GS: GlobalStateContract, T: ForEachDelegate> Layout<GS> for ForEach<GS, T> {
//     fn flexibility(&self) -> u32 {
//         unimplemented!()
//     }
//
//     fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
//         unimplemented!()
//     }
//
//     fn position_children(&mut self) {
//         unimplemented!()
//     }
// }
