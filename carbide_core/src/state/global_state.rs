use std::fmt::{Debug, Formatter};

use crate::prelude::Environment;
use crate::state::{State, StateContract, TState};
use crate::state::state_key::StateKey;
use crate::state::widget_state::WidgetState;

// The global state needs to implement clone because the widgets do, and for them to be clone
// All the generic types need to implement it as well. The global state should never in practise
// be cloned, because that would most likely be very expensive.
pub trait GlobalState: 'static + Clone + std::fmt::Debug {}

impl<T> GlobalState for T where T: 'static + Clone + std::fmt::Debug {}


#[derive(Clone)]
pub struct GState<T, GS> where T: StateContract, GS: GlobalState {
    function: fn(state: &GS) -> &T,
    function_mut: fn(state: &mut GS) -> &mut T,
    latest_value: T,
}

impl<T: StateContract, GS: GlobalState> GState<T, GS> {
    pub fn new(
        function: fn(state: &GS) -> &T,
        function_mut: fn(state: &mut GS) -> &mut T,
    ) -> Box<Self> {
        Box::new(GState {
            function,
            function_mut,
            latest_value: T::default(),
        })
    }
}

impl<T: StateContract, GS: GlobalState> State<T, GS> for GState<T, GS> {
    fn get_value_mut<'a>(&'a mut self, _: &'a mut Environment<GS>, global_state: &'a mut GS) -> &'a mut T {
        self.latest_value = (self.function_mut)(global_state).clone();

        (self.function_mut)(global_state)
    }

    fn get_value(&mut self, _: &Environment<GS>, global_state: &GS) -> &T {
        self.latest_value = (self.function)(global_state).clone();
        &self.latest_value
    }

    fn get_latest_value(&self) -> &T {
        &self.latest_value
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        &mut self.latest_value
    }

    fn get_key(&self) -> Option<&StateKey> {
        None
    }

    fn update_dependent_states(&mut self, _: &Environment<GS>) {}

    fn insert_dependent_states(&self, _: &mut Environment<GS>) {}
}

impl<T: StateContract, U: GlobalState> Debug for GState<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::GlobalState")
            .field("latest_value", &self.latest_value)
            .finish()
    }
}

impl<T: StateContract + 'static, GS: GlobalState> Into<TState<T, GS>> for Box<GState<T, GS>> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(self)
    }
}