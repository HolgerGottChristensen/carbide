use crate::prelude::GlobalState;
use serde::Serialize;
use std::fmt::Debug;
use crate::state::state::State;
use uuid::Uuid;
use crate::state::environment::Environment;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct MappedState<T, U, GS> where T: Serialize + Clone + Debug, U: Serialize + Clone + Debug, GS: GlobalState {
    id: Option<Uuid>,
    mapped_state: Box<dyn State<U, GS>>,
    map: fn(&U) -> T,
    latest_value: T,
}

impl<T: Serialize + Clone + Debug, U: Serialize + Clone + Debug, GS: GlobalState> MappedState<T, U, GS> {

    pub fn new_local(state: Box<dyn State<U, GS>>, map: fn(&U) -> T, start: T) -> MappedState<T, U, GS> {
        MappedState {
            id: Some(Uuid::new_v4()),
            mapped_state: state,
            map,
            latest_value: start
        }
    }

    pub fn new(state: Box<dyn State<U, GS>>, map: fn(&U) -> T, start: T) -> MappedState<T, U, GS> {
        MappedState {
            id: None,
            mapped_state: state,
            map,
            latest_value: start
        }
    }
}

impl<T: Serialize + Clone + Debug, U: Serialize + Clone + Debug + DeserializeOwned, GS: GlobalState> State<T, GS> for MappedState<T, U, GS> {
    fn get_value_mut(&mut self, global_state: &mut GS) -> &mut T {
        self.latest_value = (self.map)(self.mapped_state.get_value_mut(global_state));
        &mut self.latest_value
    }

    fn get_value(&mut self, global_state: &GS) -> &T {
        self.latest_value = (self.map)(self.mapped_state.get_value(global_state));
        &self.latest_value
    }

    fn get_latest_value(&self) -> &T {
        &self.latest_value
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        &mut self.latest_value
    }

    fn get_key(&self) -> Option<&String> {
        None
    }

    fn update_dependent_states(&mut self, env: &Environment<GS>) {
        env.update_local_state(&mut self.mapped_state)
    }
}
