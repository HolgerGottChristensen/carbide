use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use uuid::Uuid;

use crate::environment::environment::Environment;
use crate::state::{State, StateContract, TState};
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::state::state_key::StateKey;
use crate::state::widget_state::WidgetState;

pub struct LocalState<T> where T: StateContract {
    key: StateKey,
    value: Option<Box<dyn Any>>,
    phantom: PhantomData<T>,
}

impl<T: StateContract + 'static> LocalState<T> {
    pub fn new(value: T) -> Self {
        LocalState {
            key: StateKey::Uuid(Uuid::new_v4()),
            value: Some(Box::new(value)),
            phantom: Default::default(),
        }
    }

    pub(crate) fn key(&self) -> &StateKey {
        &self.key
    }

    pub(crate) fn value(&mut self) -> &mut Option<Box<dyn Any>> {
        &mut self.value
    }
}

impl<T: StateContract + 'static> Clone for LocalState<T> {
    fn clone(&self) -> Self {
        LocalState {
            key: self.key.clone(),
            value: None,
            phantom: Default::default(),
        }
    }
}

impl<T: StateContract + 'static> Deref for LocalState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref()
            .expect("The local state is used before it has been captured")
            .deref()
            .downcast_ref()
            .expect("Could not downcast ref for local state")
    }
}

impl<T: StateContract + 'static> DerefMut for LocalState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
            .as_mut()
            .expect("The local state is used before it has been captured")
            .deref_mut()
            .downcast_mut()
            .expect("Could not downcast ref for local state")
    }
}

impl<T: StateContract + 'static, GS: GlobalStateContract> State<T, GS> for LocalState<T> {
    fn capture_state(&mut self, env: &mut Environment<GS>, _: &GlobalStateContainer<GS>) {
        env.swap_local_state(self);
    }

    fn release_state(&mut self, env: &mut Environment<GS>) {
        env.swap_local_state(self);
    }
}

impl<T: StateContract + 'static> Debug for LocalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::LocalState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract + 'static, GS: GlobalStateContract> Into<TState<T, GS>> for Box<LocalState<T>> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(self)
    }
}