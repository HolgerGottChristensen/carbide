use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::prelude::Environment;
use crate::prelude::GlobalState;
use crate::state::{StateContract, TState};
use crate::state::state::State;
use crate::state::state_key::StateKey;
use crate::state::widget_state::WidgetState;

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
    fn get_value_mut<'a>(&'a mut self, env: &'a mut Environment<GS>, global_state: &'a mut GS) -> &'a mut T {
        self.latest_index = *self.index_state.get_value(env, global_state);
        self.latest_value = self.vec.get_value(env, global_state)[self.latest_index].clone();
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

impl<T: StateContract + 'static, GS: GlobalState> Into<TState<T, GS>> for VecState<T, GS> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract + 'static, GS: GlobalState> Into<TState<T, GS>> for Box<VecState<T, GS>> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(self)
    }
}