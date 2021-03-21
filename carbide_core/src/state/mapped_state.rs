use crate::prelude::GlobalState;
use serde::Serialize;
use std::fmt::Debug;
use crate::state::state::State;
use uuid::Uuid;
use crate::state::environment::Environment;
use serde::de::DeserializeOwned;
use crate::state::state_key::StateKey;

#[derive(Clone)]
pub struct MappedState<T, U, GS> where T: Serialize + Clone + Debug, U: Serialize + Clone + Debug, GS: GlobalState {
    id: Option<StateKey>,
    mapped_state: Box<dyn State<U, GS>>,
    map: fn(&U) -> T,
    map_back: Option<fn(U, &T) -> U>,
    latest_value: T,
}

impl<T: Serialize + Clone + Debug, U: Serialize + Clone + Debug, GS: GlobalState> MappedState<T, U, GS> {

    pub fn new_local(state: Box<dyn State<U, GS>>, map: fn(&U) -> T, start: T) -> Box<MappedState<T, U, GS>> {
        Box::new(MappedState {
            id: Some(StateKey::String(Uuid::new_v4().to_string())),
            mapped_state: state,
            map,
            map_back: None,
            latest_value: start
        })
    }

    pub fn new(state: Box<dyn State<U, GS>>, map: fn(&U) -> T, start: T) -> Box<MappedState<T, U, GS>> {
        Box::new(MappedState {
            id: None,
            mapped_state: state,
            map,
            map_back: None,
            latest_value: start
        })
    }

    pub fn map_back(mut self, f: fn(U, &T) -> U) -> Box<MappedState<T, U, GS>> {
        self.map_back = Some(f);

        Box::new(self)
    }
}

impl<T: Serialize + Clone + Debug, U: Serialize + Clone + Debug + DeserializeOwned, GS: GlobalState> State<T, GS> for MappedState<T, U, GS> {
    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut T {
        self.latest_value = (self.map)(self.mapped_state.get_value_mut(env, global_state));
        &mut self.latest_value
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &T {
        self.latest_value = (self.map)(self.mapped_state.get_value(env, global_state));
        &self.latest_value
    }

    fn get_latest_value(&self) -> &T {
        &self.latest_value
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        &mut self.latest_value
    }

    fn get_key(&self) -> Option<&StateKey> {
        if let Some(id) = &self.id {
            Some(id)
        } else {
            None
        }
    }

    fn update_dependent_states(&mut self, env: &Environment<GS>) {
        env.update_local_state(&mut self.mapped_state)
    }

    fn insert_dependent_states(&self, env: &mut Environment<GS>) {
        //Todo: If a map back function is made, we could map the value back and insert that into the environment

        /*if let Some(map_back) = &self.map_back {
            if let Some(key) = self.mapped_state.get_key() {
                let mapped_back = map_back(self.mapped_state.get_latest_value().clone(), &self.latest_value);
                env.insert_local_state_from_key_value(key, &mapped_back)
            }
        }*/
    }
}
