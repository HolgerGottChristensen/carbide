use crate::prelude::GlobalState;
use crate::state::environment::Environment;
use crate::state::state::State;
use crate::state::state_key::StateKey;
use crate::state::{TState, StateContract};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct TupleState2<T, U, GS> where T: StateContract, U: StateContract, GS: GlobalState {
    fst: TState<T, GS>,
    snd: TState<U, GS>,
    latest_value: (T, U),
}

impl<T: StateContract, U: StateContract, GS: GlobalState> TupleState2<T, U, GS> {
    pub fn new<IT, IU>(fst: IT, snd: IU) -> Box<TupleState2<T, U, GS>>
        where
            IT: Into<TState<T, GS>>,
            IU: Into<TState<U, GS>>
    {
        let fst = fst.into();
        let snd = snd.into();

        Box::new(TupleState2 {
            fst: fst.clone(),
            snd: snd.clone(),
            latest_value: (fst.get_latest_value().clone(), snd.get_latest_value().clone()),
        })
    }
}

impl<T: StateContract, U: StateContract, GS: GlobalState> From<(TState<T, GS>, TState<U, GS>)> for TupleState2<T, U, GS> {
    fn from((first, second): (TState<T, GS>, TState<U, GS>)) -> Self {
        TupleState2 {
            fst: first.clone(),
            snd: second.clone(),
            latest_value: (first.get_latest_value().clone(), second.get_latest_value().clone()),
        }
    }
}

impl<T: StateContract + 'static, U: StateContract + 'static, GS: GlobalState> Into<TState<(T, U), GS>> for Box<TupleState2<T, U, GS>> {
    fn into(self) -> TState<(T, U), GS> {
        WidgetState::new(self)
    }
}

impl<T: StateContract, U: StateContract, GS: GlobalState> State<(T, U), GS> for TupleState2<T, U, GS> {
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
