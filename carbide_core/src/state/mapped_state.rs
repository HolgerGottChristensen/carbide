// use std::fmt::{Debug, Formatter};
// use std::ops::{Deref, DerefMut};
//
// use crate::prelude::Environment;
// use crate::prelude::GlobalStateContract;
// use crate::state::{StateContract, TState};
// use crate::state::global_state::GlobalStateContainer;
// use crate::state::state::State;
// use crate::state::widget_state::WidgetState;
//
// #[derive(Clone)]
// pub struct MappedState<T, U, GS> where T: StateContract, U: StateContract, GS: GlobalStateContract {
//     mapped_state: TState<U, GS>,
//     map: fn(&U) -> T,
//     map_back: Option<fn(U, &T) -> U>,
//     value: T,
// }
//
// impl<T: StateContract + Default, U: StateContract, GS: GlobalStateContract> MappedState<T, U, GS> {
//     pub fn new<V: Into<TState<U, GS>>>(state: V, map: fn(&U) -> T) -> Box<MappedState<T, U, GS>> {
//         Box::new(MappedState {
//             mapped_state: state.into(),
//             map,
//             map_back: None,
//             value: T::default(),
//         })
//     }
//
//     pub fn map_back(mut self, f: fn(U, &T) -> U) -> Box<MappedState<T, U, GS>> {
//         self.map_back = Some(f);
//
//         Box::new(self)
//     }
// }
//
// impl<T: StateContract, U: StateContract, GS: GlobalStateContract> Deref for MappedState<T, U, GS> {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }
//
// impl<T: StateContract, U: StateContract, GS: GlobalStateContract> DerefMut for MappedState<T, U, GS> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.value
//     }
// }
//
// impl<T: StateContract, U: StateContract, GS: GlobalStateContract> State<T, GS> for MappedState<T, U, GS> {
//     fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
//         self.mapped_state.capture_state(env, global_state);
//         self.value = (self.map)(&*self.mapped_state)
//     }
//
//     fn release_state(&mut self, env: &mut Environment<GS>) {
//         self.mapped_state.release_state(env);
//     }
// }
//
// impl<T: StateContract, U: StateContract, GS: GlobalStateContract> Debug for MappedState<T, U, GS> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("State::MappedState")
//             .field("value", self.deref())
//             .finish()
//     }
// }
//
// impl<T: StateContract + 'static, U: StateContract + 'static, GS: GlobalStateContract> Into<TState<T, GS>> for Box<MappedState<T, U, GS>> {
//     fn into(self) -> TState<T, GS> {
//         WidgetState::new(self)
//     }
// }
