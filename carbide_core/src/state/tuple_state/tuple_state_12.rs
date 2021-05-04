use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::prelude::GlobalState;
use crate::state::environment::Environment;
use crate::state::state::State;
use crate::state::state_key::StateKey;
use crate::state::TState;

#[derive(Clone)]
pub struct TupleState12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          T8: Serialize + Clone + Debug + DeserializeOwned,
          T9: Serialize + Clone + Debug + DeserializeOwned,
          T10: Serialize + Clone + Debug + DeserializeOwned,
          T11: Serialize + Clone + Debug + DeserializeOwned,
          T12: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {
    first: Box<dyn State<T1, GS>>,
    second: Box<dyn State<T2, GS>>,
    third: Box<dyn State<T3, GS>>,
    fourth: Box<dyn State<T4, GS>>,
    fifth: Box<dyn State<T5, GS>>,
    sixth: Box<dyn State<T6, GS>>,
    seventh: Box<dyn State<T7, GS>>,
    eighth: Box<dyn State<T8, GS>>,
    ninth: Box<dyn State<T9, GS>>,
    tenth: Box<dyn State<T10, GS>>,
    eleventh: Box<dyn State<T11, GS>>,
    twelfth: Box<dyn State<T12, GS>>,
    latest_value: (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12),
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS> TupleState12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          T8: Serialize + Clone + Debug + DeserializeOwned,
          T9: Serialize + Clone + Debug + DeserializeOwned,
          T10: Serialize + Clone + Debug + DeserializeOwned,
          T11: Serialize + Clone + Debug + DeserializeOwned,
          T12: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {
    pub fn new<IT1, IT2, IT3, IT4, IT5, IT6, IT7, IT8, IT9, IT10, IT11, IT12>(
        first: IT1,
        second: IT2,
        third: IT3,
        fourth: IT4,
        fifth: IT5,
        sixth: IT6,
        seventh: IT7,
        eighth: IT8,
        ninth: IT9,
        tenth: IT10,
        eleventh: IT11,
        twelfth: IT12,
    ) -> Box<TupleState12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS>>
        where
            IT1: Into<TState<T1, GS>>,
            IT2: Into<TState<T2, GS>>,
            IT3: Into<TState<T3, GS>>,
            IT4: Into<TState<T4, GS>>,
            IT5: Into<TState<T5, GS>>,
            IT6: Into<TState<T6, GS>>,
            IT7: Into<TState<T7, GS>>,
            IT8: Into<TState<T8, GS>>,
            IT9: Into<TState<T9, GS>>,
            IT10: Into<TState<T10, GS>>,
            IT11: Into<TState<T11, GS>>,
            IT12: Into<TState<T12, GS>>,
    {
        let first = first.into();
        let second = second.into();
        let third = third.into();
        let fourth = fourth.into();
        let fifth = fifth.into();
        let sixth = sixth.into();
        let seventh = seventh.into();
        let eighth = eighth.into();
        let ninth = ninth.into();
        let tenth = tenth.into();
        let eleventh = eleventh.into();
        let twelfth = twelfth.into();

        Box::new(TupleState12 {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            fifth: fifth.clone(),
            sixth: sixth.clone(),
            seventh: seventh.clone(),
            eighth: eighth.clone(),
            ninth: ninth.clone(),
            tenth: tenth.clone(),
            eleventh: eleventh.clone(),
            twelfth: twelfth.clone(),
            latest_value: (
                first.get_latest_value().clone(),
                second.get_latest_value().clone(),
                third.get_latest_value().clone(),
                fourth.get_latest_value().clone(),
                fifth.get_latest_value().clone(),
                sixth.get_latest_value().clone(),
                seventh.get_latest_value().clone(),
                eighth.get_latest_value().clone(),
                ninth.get_latest_value().clone(),
                tenth.get_latest_value().clone(),
                eleventh.get_latest_value().clone(),
                twelfth.get_latest_value().clone(),
            ),
        })
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS> From<(
    Box<dyn State<T1, GS>>,
    Box<dyn State<T2, GS>>,
    Box<dyn State<T3, GS>>,
    Box<dyn State<T4, GS>>,
    Box<dyn State<T5, GS>>,
    Box<dyn State<T6, GS>>,
    Box<dyn State<T7, GS>>,
    Box<dyn State<T8, GS>>,
    Box<dyn State<T9, GS>>,
    Box<dyn State<T10, GS>>,
    Box<dyn State<T11, GS>>,
    Box<dyn State<T12, GS>>,
)> for TupleState12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          T8: Serialize + Clone + Debug + DeserializeOwned,
          T9: Serialize + Clone + Debug + DeserializeOwned,
          T10: Serialize + Clone + Debug + DeserializeOwned,
          T11: Serialize + Clone + Debug + DeserializeOwned,
          T12: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {
    fn from((first, second, third, fourth, fifth, sixth, seventh, eighth, ninth, tenth, eleventh, twelfth): (
        Box<dyn State<T1, GS>>,
        Box<dyn State<T2, GS>>,
        Box<dyn State<T3, GS>>,
        Box<dyn State<T4, GS>>,
        Box<dyn State<T5, GS>>,
        Box<dyn State<T6, GS>>,
        Box<dyn State<T7, GS>>,
        Box<dyn State<T8, GS>>,
        Box<dyn State<T9, GS>>,
        Box<dyn State<T10, GS>>,
        Box<dyn State<T11, GS>>,
        Box<dyn State<T12, GS>>,
    )) -> Self {
        TupleState12 {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            fifth: fifth.clone(),
            sixth: sixth.clone(),
            seventh: seventh.clone(),
            eighth: eighth.clone(),
            ninth: ninth.clone(),
            tenth: tenth.clone(),
            eleventh: eleventh.clone(),
            twelfth: twelfth.clone(),
            latest_value: (
                first.get_latest_value().clone(),
                second.get_latest_value().clone(),
                third.get_latest_value().clone(),
                fourth.get_latest_value().clone(),
                fifth.get_latest_value().clone(),
                sixth.get_latest_value().clone(),
                seventh.get_latest_value().clone(),
                eighth.get_latest_value().clone(),
                ninth.get_latest_value().clone(),
                tenth.get_latest_value().clone(),
                eleventh.get_latest_value().clone(),
                twelfth.get_latest_value().clone(),
            ),
        }
    }
}


impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS> State<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12), GS> for TupleState12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, GS>
    where T1: Serialize + Clone + Debug + DeserializeOwned,
          T2: Serialize + Clone + Debug + DeserializeOwned,
          T3: Serialize + Clone + Debug + DeserializeOwned,
          T4: Serialize + Clone + Debug + DeserializeOwned,
          T5: Serialize + Clone + Debug + DeserializeOwned,
          T6: Serialize + Clone + Debug + DeserializeOwned,
          T7: Serialize + Clone + Debug + DeserializeOwned,
          T8: Serialize + Clone + Debug + DeserializeOwned,
          T9: Serialize + Clone + Debug + DeserializeOwned,
          T10: Serialize + Clone + Debug + DeserializeOwned,
          T11: Serialize + Clone + Debug + DeserializeOwned,
          T12: Serialize + Clone + Debug + DeserializeOwned,
          GS: GlobalState {

    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12) {
        self.latest_value = (
            self.first.get_value_mut(env, global_state).clone(),
            self.second.get_value_mut(env, global_state).clone(),
            self.third.get_value_mut(env, global_state).clone(),
            self.fourth.get_value_mut(env, global_state).clone(),
            self.fifth.get_value_mut(env, global_state).clone(),
            self.sixth.get_value_mut(env, global_state).clone(),
            self.seventh.get_value_mut(env, global_state).clone(),
            self.eighth.get_value_mut(env, global_state).clone(),
            self.ninth.get_value_mut(env, global_state).clone(),
            self.tenth.get_value_mut(env, global_state).clone(),
            self.eleventh.get_value_mut(env, global_state).clone(),
            self.twelfth.get_value_mut(env, global_state).clone(),
        );
        &mut self.latest_value
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12) {
        self.latest_value = (
            self.first.get_value(env, global_state).clone(),
            self.second.get_value(env, global_state).clone(),
            self.third.get_value(env, global_state).clone(),
            self.fourth.get_value(env, global_state).clone(),
            self.fifth.get_value(env, global_state).clone(),
            self.sixth.get_value(env, global_state).clone(),
            self.seventh.get_value(env, global_state).clone(),
            self.eighth.get_value(env, global_state).clone(),
            self.ninth.get_value(env, global_state).clone(),
            self.tenth.get_value(env, global_state).clone(),
            self.eleventh.get_value(env, global_state).clone(),
            self.twelfth.get_value(env, global_state).clone(),
        );
        &self.latest_value
    }

    fn get_latest_value(&self) -> &(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12) {
        &self.latest_value
    }

    fn get_latest_value_mut(&mut self) -> &mut (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12) {
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
        env.update_local_state(&mut self.eighth);
        env.update_local_state(&mut self.ninth);
        env.update_local_state(&mut self.tenth);
        env.update_local_state(&mut self.eleventh);
        env.update_local_state(&mut self.twelfth);
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

        if let Some(eighth_key) = self.eighth.get_key() {
            env.insert_local_state_from_key_value(eighth_key, &self.latest_value.7);
        }

        if let Some(ninth_key) = self.ninth.get_key() {
            env.insert_local_state_from_key_value(ninth_key, &self.latest_value.8);
        }

        if let Some(tenth_key) = self.tenth.get_key() {
            env.insert_local_state_from_key_value(tenth_key, &self.latest_value.9);
        }

        if let Some(eleventh_key) = self.eleventh.get_key() {
            env.insert_local_state_from_key_value(eleventh_key, &self.latest_value.10);
        }

        if let Some(twelfth_key) = self.twelfth.get_key() {
            env.insert_local_state_from_key_value(twelfth_key, &self.latest_value.11);
        }
    }
}
