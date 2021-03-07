use crate::prelude::GlobalState;
use serde::Serialize;
use std::fmt::Debug;
use crate::state::state::State;
use crate::state::environment::Environment;
use serde::de::DeserializeOwned;
use crate::state::state_key::StateKey;

#[derive(Clone)]
pub struct TupleState7<T1, T2, T3, T4, T5, T6, T7, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {
    first: Box<dyn State<T1, GS>>,
    second: Box<dyn State<T2, GS>>,
    third: Box<dyn State<T3, GS>>,
    fourth: Box<dyn State<T4, GS>>,
    fifth: Box<dyn State<T5, GS>>,
    sixth: Box<dyn State<T6, GS>>,
    seventh: Box<dyn State<T7, GS>>,
    latest_value: (T1, T2, T3, T4, T5, T6, T7),
}

impl<T1, T2, T3, T4, T5, T6, T7, GS> TupleState7<T1, T2, T3, T4, T5, T6, T7, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {

    pub fn new(first: Box<dyn State<T1, GS>>,
               second: Box<dyn State<T2, GS>>,
               third: Box<dyn State<T3, GS>>,
               fourth: Box<dyn State<T4, GS>>,
               fifth: Box<dyn State<T5, GS>>,
               sixth: Box<dyn State<T6, GS>>,
               seventh: Box<dyn State<T7, GS>>,
    ) -> Box<TupleState7<T1, T2, T3, T4, T5, T6, T7, GS>> {
        Box::new(TupleState7 {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            fifth: fifth.clone(),
            sixth: sixth.clone(),
            seventh: seventh.clone(),
            latest_value: (
                first.get_latest_value().clone(),
                second.get_latest_value().clone(),
                third.get_latest_value().clone(),
                fourth.get_latest_value().clone(),
                fifth.get_latest_value().clone(),
                sixth.get_latest_value().clone(),
                seventh.get_latest_value().clone(),
            ),
        })
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, GS> From<(
    Box<dyn State<T1, GS>>,
    Box<dyn State<T2, GS>>,
    Box<dyn State<T3, GS>>,
    Box<dyn State<T4, GS>>,
    Box<dyn State<T5, GS>>,
    Box<dyn State<T6, GS>>,
    Box<dyn State<T7, GS>>,
)> for TupleState7<T1, T2, T3, T4, T5, T6, T7, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {
    fn from((first, second, third, fourth, fifth, sixth, seventh): (Box<dyn State<T1, GS>>, Box<dyn State<T2, GS>>, Box<dyn State<T3, GS>>, Box<dyn State<T4, GS>>, Box<dyn State<T5, GS>>, Box<dyn State<T6, GS>>, Box<dyn State<T7, GS>>)) -> Self {
        TupleState7 {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            fifth: fifth.clone(),
            sixth: sixth.clone(),
            seventh: seventh.clone(),
            latest_value: (
                first.get_latest_value().clone(),
                second.get_latest_value().clone(),
                third.get_latest_value().clone(),
                fourth.get_latest_value().clone(),
                fifth.get_latest_value().clone(),
                sixth.get_latest_value().clone(),
                seventh.get_latest_value().clone(),
            ),
        }
    }
}


impl<T1, T2, T3, T4, T5, T6, T7, GS> State<(T1, T2, T3, T4, T5, T6, T7), GS> for TupleState7<T1, T2, T3, T4, T5, T6, T7, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {

    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut (T1, T2, T3, T4, T5, T6, T7) {
        self.latest_value = (
            self.first.get_value_mut(env, global_state).clone(),
            self.second.get_value_mut(env, global_state).clone(),
            self.third.get_value_mut(env, global_state).clone(),
            self.fourth.get_value_mut(env, global_state).clone(),
            self.fifth.get_value_mut(env, global_state).clone(),
            self.sixth.get_value_mut(env, global_state).clone(),
            self.seventh.get_value_mut(env, global_state).clone(),
        );
        &mut self.latest_value
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &(T1, T2, T3, T4, T5, T6, T7) {
        self.latest_value = (
            self.first.get_value(env, global_state).clone(),
            self.second.get_value(env, global_state).clone(),
            self.third.get_value(env, global_state).clone(),
            self.fourth.get_value(env, global_state).clone(),
            self.fifth.get_value(env, global_state).clone(),
            self.sixth.get_value(env, global_state).clone(),
            self.seventh.get_value(env, global_state).clone(),
        );
        &self.latest_value
    }

    fn get_latest_value(&self) -> &(T1, T2, T3, T4, T5, T6, T7) {
        &self.latest_value
    }

    fn get_latest_value_mut(&mut self) -> &mut (T1, T2, T3, T4, T5, T6, T7) {
        &mut self.latest_value
    }

    fn get_key(&self) -> Option<&StateKey> {
        None
    }

    fn update_dependent_states(&mut self, env: &Environment<GS>) {
        env.update_local_state(&mut self.first);
        env.update_local_state(&mut self.second);
        env.update_local_state(&mut self.third);
        env.update_local_state(&mut self.fourth);
        env.update_local_state(&mut self.fifth);
        env.update_local_state(&mut self.sixth);
        env.update_local_state(&mut self.seventh);
    }

    fn insert_dependent_states(&self, env: &mut Environment<GS>) {

        if let Some(fst_key) = self.first.get_key() {
            env.insert_local_state_from_key_value(fst_key, &self.latest_value.0);
        }

        if let Some(snd_key) = self.second.get_key() {
            env.insert_local_state_from_key_value(snd_key, &self.latest_value.1);
        }

        if let Some(third_key) = self.third.get_key() {
            env.insert_local_state_from_key_value(third_key, &self.latest_value.2);
        }

        if let Some(fourth_key) = self.fourth.get_key() {
            env.insert_local_state_from_key_value(fourth_key, &self.latest_value.3);
        }

        if let Some(fifth_key) = self.fifth.get_key() {
            env.insert_local_state_from_key_value(fifth_key, &self.latest_value.4);
        }

        if let Some(sixth_key) = self.sixth.get_key() {
            env.insert_local_state_from_key_value(sixth_key, &self.latest_value.5);
        }

        if let Some(seventh_key) = self.seventh.get_key() {
            env.insert_local_state_from_key_value(seventh_key, &self.latest_value.6);
        }
    }
}
