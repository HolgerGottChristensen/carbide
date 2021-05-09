use crate::prelude::Environment;
use crate::prelude::GlobalState;
use crate::state::{StateContract, TState};
use crate::state::state::State;
use crate::state::state_key::StateKey;
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct TupleState4<T1, T2, T3, T4, GS>
    where T1: StateContract,
          T2: StateContract,
          T3: StateContract,
          T4: StateContract,
          GS: GlobalState {
    first: TState<T1, GS>,
    second: TState<T2, GS>,
    third: TState<T3, GS>,
    fourth: TState<T4, GS>,
    latest_value: (T1, T2, T3, T4),
}

impl<T1, T2, T3, T4, GS> TupleState4<T1, T2, T3, T4, GS>
    where T1: StateContract,
          T2: StateContract,
          T3: StateContract,
          T4: StateContract,
          GS: GlobalState {
    pub fn new<IT1, IT2, IT3, IT4>(first: IT1, second: IT2, third: IT3, fourth: IT4) -> Box<TupleState4<T1, T2, T3, T4, GS>>
        where
            IT1: Into<TState<T1, GS>>,
            IT2: Into<TState<T2, GS>>,
            IT3: Into<TState<T3, GS>>,
            IT4: Into<TState<T4, GS>>,
    {
        let first = first.into();
        let second = second.into();
        let third = third.into();
        let fourth = fourth.into();

        Box::new(TupleState4 {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            latest_value: (first.get_latest_value().clone(), second.get_latest_value().clone(), third.get_latest_value().clone(), fourth.get_latest_value().clone()),
        })
    }
}

/*impl<T1, T2, T3, T4, GS> From<(Box<dyn State<T1, GS>>, Box<dyn State<T2, GS>>, Box<dyn State<T3, GS>>, Box<dyn State<T4, GS>>)> for TupleState4<T1, T2, T3, T4, GS>
    where T1: StateContract,
          T2: StateContract,
          T3: StateContract,
          T4: StateContract,
          GS: GlobalState {
    fn from((first, second, third, fourth): (Box<dyn State<T1, GS>>, Box<dyn State<T2, GS>>, Box<dyn State<T3, GS>>, Box<dyn State<T4, GS>>)) -> Self {
        TupleState4 {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            latest_value: (first.get_latest_value().clone(), second.get_latest_value().clone(), third.get_latest_value().clone(), fourth.get_latest_value().clone()),
        }
    }
}*/

impl<T1, T2, T3, T4, GS> Into<TState<(T1, T2, T3, T4), GS>> for Box<TupleState4<T1, T2, T3, T4, GS>>
    where T1: StateContract + 'static,
          T2: StateContract + 'static,
          T3: StateContract + 'static,
          T4: StateContract + 'static,
          GS: GlobalState {
    fn into(self) -> TState<(T1, T2, T3, T4), GS> {
        WidgetState::new(self)
    }
}

impl<T1, T2, T3, T4, GS> State<(T1, T2, T3, T4), GS> for TupleState4<T1, T2, T3, T4, GS>
    where T1: StateContract,
          T2: StateContract,
          T3: StateContract,
          T4: StateContract,
          GS: GlobalState {

    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut (T1, T2, T3, T4) {
        self.latest_value = (self.first.get_value_mut(env, global_state).clone(), self.second.get_value_mut(env, global_state).clone(), self.third.get_value_mut(env, global_state).clone(), self.fourth.get_value_mut(env, global_state).clone());
        &mut self.latest_value
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &(T1, T2, T3, T4) {
        self.latest_value = (self.first.get_value(env, global_state).clone(), self.second.get_value(env, global_state).clone(), self.third.get_value(env, global_state).clone(), self.fourth.get_value(env, global_state).clone());
        &self.latest_value
    }

    fn get_latest_value(&self) -> &(T1, T2, T3, T4) {
        &self.latest_value
    }

    fn get_latest_value_mut(&mut self) -> &mut (T1, T2, T3, T4) {
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
    }
}
