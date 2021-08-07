use crate::event::event::Event;
use crate::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::prelude::*;
use crate::state::global_state::GlobalStateContainer;

// /// A basic, non-interactive rectangle shape widget.
// #[derive(Debug, Clone, Widget)]
// #[event(process_keyboard_event, process_mouse_event, process_other_event)]
// pub struct EnvUpdating<GS> where GS: GlobalStateContract {
//     id: Uuid,
//     child: Box<dyn Widget<GS>>,
//     position: Point,
//     dimension: Dimensions,
//     envs_to_update: Vec<EnvironmentStateContainer<GS>>,
// }

#[derive(Debug, Clone)]
pub enum EnvironmentStateContainer {
    String { key: String, value: StringState },
    U32 { key: String, value: U32State },
    F64 { key: String, value: F64State },
    Color { key: EnvironmentColor, value: ColorState },
    FontSize { key: EnvironmentFontSize, value: U32State },
    I32 { key: String, value: I32State },
}
//
// impl<GS: GlobalStateContract> EnvUpdating<GS> {
//     pub fn new(child: Box<dyn Widget<GS>>) -> Box<EnvUpdating<GS>> {
//         Box::new(EnvUpdating {
//             id: Uuid::new_v4(),
//             child,
//             position: [0.0, 0.0],
//             dimension: [100.0, 100.0],
//             envs_to_update: vec![],
//         })
//     }
//
//     pub fn add(&mut self, env_to_update: EnvironmentStateContainer<GS>) {
//         self.envs_to_update.push(env_to_update);
//     }
//
//     fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
//         self.insert_into_env(env);
//
//         self.process_keyboard_event_default(event, env, global_state);
//
//         self.remove_from_env(env);
//     }
//
//     fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
//         self.insert_into_env(env);
//
//         self.process_mouse_event_default(event, consumed, env, global_state);
//
//         self.remove_from_env(env);
//     }
//
//     fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
//         self.insert_into_env(env);
//
//         self.process_other_event_default(event, env, global_state);
//
//         self.remove_from_env(env);
//     }
//
//     fn remove_from_env(&self, env: &mut Environment<GS>) {
//         for _ in &self.envs_to_update {
//             env.pop()
//         }
//     }
//
//     fn insert_into_env(&mut self, env: &mut Environment<GS>) {
//         for env_to_update in &mut self.envs_to_update {
//             match env_to_update {
//                 EnvironmentStateContainer::String { key, value } => {
//                     let to_update = (&***value).clone();
//
//                     env.push(EnvironmentVariable::String { key: key.clone(), value: to_update })
//                 }
//                 EnvironmentStateContainer::U32 { key, value } => {
//                     let to_update = **value.clone();
//
//                     env.push(EnvironmentVariable::U32 { key: key.clone(), value: to_update })
//                 }
//                 EnvironmentStateContainer::F64 { key, value } => {
//                     let to_update = **value.clone();
//
//                     env.push(EnvironmentVariable::F64 { key: key.clone(), value: to_update })
//                 }
//                 EnvironmentStateContainer::Color { key, value } => {
//                     let to_update = *value.clone();
//
//                     env.push(EnvironmentVariable::Color { key: key.clone(), value: to_update })
//                 }
//                 EnvironmentStateContainer::FontSize { key, value } => {
//                     let to_update = *value.clone();
//
//                     env.push(EnvironmentVariable::FontSize { key: key.clone(), value: to_update })
//                 }
//                 EnvironmentStateContainer::I32 { key, value } => {
//                     let to_update = **value.clone();
//
//                     env.push(EnvironmentVariable::I32 { key: key.clone(), value: to_update })
//                 }
//             }
//         }
//     }
// }
//
//
// impl<GS: GlobalStateContract> Layout<GS> for EnvUpdating<GS> {
//     fn flexibility(&self) -> u32 {
//         self.child.flexibility()
//     }
//
//     fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
//         self.dimension = self.child.calculate_size(requested_size, env);
//         self.dimension
//     }
//
//     fn position_children(&mut self) {
//         let positioning = BasicLayouter::Center.position();
//         let position = self.position;
//         let dimension = self.dimension;
//
//         positioning(position, dimension, &mut self.child);
//         self.child.position_children();
//     }
// }
//
// impl<GS: GlobalStateContract> CommonWidget<GS> for EnvUpdating<GS> {
//     fn get_id(&self) -> Uuid {
//         self.id
//     }
//
//     fn set_id(&mut self, id: Uuid) {
//         self.id = id;
//     }
//
//     fn get_flag(&self) -> Flags {
//         Flags::EMPTY
//     }
//
//     fn get_children(&self) -> WidgetIter<GS> {
//         if self.child.get_flag() == Flags::PROXY {
//             self.child.get_children()
//         } else {
//             WidgetIter::single(&self.child)
//         }
//     }
//
//     fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
//         if self.child.get_flag() == Flags::PROXY {
//             self.child.get_children_mut()
//         } else {
//             WidgetIterMut::single(&mut self.child)
//         }
//     }
//
//     fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
//         WidgetIterMut::single(&mut self.child)
//     }
//
//     fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
//         WidgetIterMut::single(&mut self.child)
//     }
//
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
// impl<GS: GlobalStateContract> Render<GS> for EnvUpdating<GS> {
//     fn get_primitives(&mut self, env: &mut Environment<GS>) -> Vec<Primitive> {
//         let prims = self.child.get_primitives(env);
//         return prims;
//     }
// }
//
//
// impl<GS: GlobalStateContract> WidgetExt<GS> for EnvUpdating<GS> {}