use crate::prelude::GlobalState;
use serde::Serialize;
use std::fmt::Debug;
use crate::state::state::State;
use crate::state::environment::Environment;
use serde::de::DeserializeOwned;
use crate::state::state_key::StateKey;

#[derive(Clone)]
pub struct TupleState2<T, U, GS> where T: Serialize + Clone + Debug + DeserializeOwned, U: Serialize + Clone + Debug + DeserializeOwned, GS: GlobalState {
    fst: Box<dyn State<T, GS>>,
    snd: Box<dyn State<U, GS>>,
    latest_value: (T, U),
}

impl<T: Serialize + Clone + Debug + DeserializeOwned, U: Serialize + Clone + Debug + DeserializeOwned, GS: GlobalState> TupleState2<T, U, GS> {

    pub fn new(fst: Box<dyn State<T, GS>>, snd: Box<dyn State<U, GS>>) -> Box<TupleState2<T, U, GS>> {
        Box::new(TupleState2 {
            fst: fst.clone(),
            snd: snd.clone(),
            latest_value: (fst.get_latest_value().clone(), snd.get_latest_value().clone()),
        })
    }
}

impl<T: Serialize + Clone + Debug + DeserializeOwned, U: Serialize + Clone + Debug + DeserializeOwned, GS: GlobalState> From<(Box<dyn State<T, GS>>, Box<dyn State<U, GS>>)> for TupleState2<T, U, GS> {
    fn from((first, second): (Box<dyn State<T, GS>>, Box<dyn State<U, GS>>)) -> Self {
        TupleState2 {
            fst: first.clone(),
            snd: second.clone(),
            latest_value: (first.get_latest_value().clone(), second.get_latest_value().clone()),
        }
    }
}

impl<T: Serialize + Clone + Debug + DeserializeOwned, U: Serialize + Clone + Debug + DeserializeOwned, GS: GlobalState> State<(T, U), GS> for TupleState2<T, U, GS> {
    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut (T, U) {
        self.latest_value = (self.fst.get_value_mut(env, global_state).clone(), self.snd.get_value_mut(env, global_state).clone());
        &mut self.latest_value
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &(T, U) {
        self.latest_value = (self.fst.get_value(env, global_state).clone(), self.snd.get_value(env, global_state).clone());
        &self.latest_value
    }

    fn get_latest_value(&self) -> &(T, U) {
        &self.latest_value
    }

    fn get_latest_value_mut(&mut self) -> &mut (T, U) {
        &mut self.latest_value
    }

    fn get_key(&self) -> Option<&StateKey> {
        None
    }

    fn update_dependent_states(&mut self, env: &Environment<GS>) {
        env.update_local_state(&mut self.fst);
        env.update_local_state(&mut self.snd);
    }

    fn insert_dependent_states(&self, env: &mut Environment<GS>) {

        if let Some(fst_key) = self.fst.get_key() {
            env.insert_local_state_from_key_value(fst_key, &self.latest_value.0);
        }

        if let Some(snd_key) = self.snd.get_key() {
            env.insert_local_state_from_key_value(snd_key, &self.latest_value.1);
        }
    }
}
