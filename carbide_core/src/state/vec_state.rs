use crate::prelude::GlobalState;
use serde::Serialize;
use std::fmt::Debug;
use crate::state::state::State;
use uuid::Uuid;
use crate::state::environment::Environment;
use serde::de::DeserializeOwned;
use crate::state::state_key::StateKey;

#[derive(Clone)]
pub struct VecState<T, GS> where T: Serialize + Clone + Debug, GS: GlobalState {
    id: Option<StateKey>,
    index_state: Box<dyn State<usize, GS>>,
    vec: Box<dyn State<Vec<T>, GS>>,
    latest_index: usize,
    latest_value: T,
}

impl<T: Serialize + Clone + Debug, GS: GlobalState> VecState<T, GS> {

    pub fn new_local(state: Box<dyn State<Vec<T>, GS>>, index: Box<dyn State<usize, GS>>, start: T) -> Box<VecState<T, GS>> {
        Box::new(VecState {
            id: Some(StateKey::String(Uuid::new_v4().to_string())),
            index_state: index,
            vec: state,
            latest_index: 0,
            latest_value: start
        })
    }

    pub fn new(state: Box<dyn State<Vec<T>, GS>>, index: Box<dyn State<usize, GS>>, start: T) -> Box<VecState<T, GS>> {
        Box::new(VecState {
            id: None,
            index_state: index,
            vec: state,
            latest_index: 0,
            latest_value: start
        })
    }
}

impl<T: Serialize + Clone + Debug + DeserializeOwned, GS: GlobalState> State<T, GS> for VecState<T, GS> {
    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut T {
        self.latest_index = *self.index_state.get_value(env, global_state);
        self.latest_value = self.vec.get_value(env, global_state)[self.latest_index].clone();
        println!("Get value mut returned: {:?}, with index: {}", self.latest_value, self.latest_index);
        &mut self.vec.get_value_mut(env, global_state)[self.latest_index]
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &T {
        self.latest_index = *self.index_state.get_value(env, global_state);
        self.latest_value = self.vec.get_value(env, global_state)[self.latest_index].clone();
        &self.vec.get_value(env, global_state)[self.latest_index]
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
        env.update_local_state(&mut self.vec);
        env.update_local_state(&mut self.index_state);
    }

    fn insert_dependent_states(&self, env: &mut Environment<GS>) {
        env.insert_local_state(&self.vec)
    }
}
