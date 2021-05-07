pub use crate::state::State;
use crate::widget::{GlobalState, CommonState};
use crate::{Serialize, DeserializeOwned};
use std::fmt::Debug;
use crate::prelude::state_key::StateKey;
use crate::prelude::Environment;
use crate::export::Formatter;
use std::fmt;

pub struct WidgetState<T, GS>(Box<dyn State<T, GS>>);

impl<T: Serialize + Clone + Debug + Default, GS: GlobalState> WidgetState<T, GS> {
    pub fn new(item: Box<dyn State<T, GS>>) -> WidgetState<T, GS> {
        WidgetState(item)
    }
}

impl<T: Serialize + Clone + Debug + Default + 'static, GS: GlobalState> Default for WidgetState<T, GS> {
    fn default() -> Self {
        Self(Box::new(CommonState::new(&T::default())))
    }
}

impl<T: Serialize + Clone + Debug + Default, GS: GlobalState> Debug for WidgetState<T, GS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Serialize + Clone + Debug + Default, GS: GlobalState> Clone for WidgetState<T, GS> {
    fn clone(&self) -> Self {
        WidgetState (self.0.clone())
    }
}

impl<T: Serialize + Clone + Debug + DeserializeOwned + Default, GS: GlobalState> Into<WidgetState<T, GS>> for Box<dyn State<T, GS>> {
    fn into(self) -> WidgetState<T, GS> {
        WidgetState(self)
    }
}


impl<T: Serialize + Clone + Debug + DeserializeOwned + Default, GS: GlobalState> State<T, GS> for WidgetState<T, GS> {
    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut T {
        self.0.get_value_mut(env, global_state)
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &T {
        self.0.get_value(env, global_state)
    }

    fn get_latest_value(&self) -> &T {
        self.0.get_latest_value()
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        self.0.get_latest_value_mut()
    }

    fn get_key(&self) -> Option<&StateKey> {
        self.0.get_key()
    }

    fn update_dependent_states(&mut self, env: &Environment<GS>) {
        self.0.update_dependent_states(env)
    }

    fn insert_dependent_states(&self, env: &mut Environment<GS>) {
        self.0.insert_dependent_states(env)
    }
}