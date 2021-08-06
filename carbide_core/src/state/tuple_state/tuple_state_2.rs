use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::prelude::Environment;
use crate::prelude::GlobalStateContract;
use crate::state::{StateContract, TState};
use crate::state::global_state::GlobalStateContainer;
use crate::state::state::State;
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct TupleState2<T, U, GS> where T: StateContract, U: StateContract, GS: GlobalStateContract {
    state: (TState<T, GS>, TState<U, GS>),
}

impl<T: StateContract, U: StateContract, GS: GlobalStateContract> TupleState2<T, U, GS> {
    pub fn new<IT, IU>(fst: IT, snd: IU) -> Box<TupleState2<T, U, GS>>
        where
            IT: Into<TState<T, GS>>,
            IU: Into<TState<U, GS>>
    {
        let fst = fst.into();
        let snd = snd.into();

        Box::new(TupleState2 {
            state: (fst, snd)
        })
    }
}

impl<T: StateContract, U: StateContract, GS: GlobalStateContract> Deref for TupleState2<T, U, GS> {
    type Target = (TState<T, GS>, TState<U, GS>);

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T: StateContract, U: StateContract, GS: GlobalStateContract> DerefMut for TupleState2<T, U, GS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<T: StateContract, U: StateContract, GS: GlobalStateContract> State<(TState<T, GS>, TState<U, GS>), GS> for TupleState2<T, U, GS> {
    fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.state.0.capture_state(env, global_state);
        self.state.1.capture_state(env, global_state);
    }

    fn release_state(&mut self, env: &mut Environment<GS>) {
        self.state.0.release_state(env);
        self.state.1.release_state(env);
    }
}

impl<T: StateContract, U: StateContract, GS: GlobalStateContract> Debug for TupleState2<T, U, GS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::TupleState2")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract + 'static, U: StateContract + 'static, GS: GlobalStateContract> Into<TState<(TState<T, GS>, TState<U, GS>), GS>> for Box<TupleState2<T, U, GS>> {
    fn into(self) -> TState<(TState<T, GS>, TState<U, GS>), GS> {
        WidgetState::new(self)
    }
}