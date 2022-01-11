use std::fmt::Debug;

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{MapRev, State};
use crate::state::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct FieldState<FROM, TO>
    where
        FROM: StateContract + 'static,
        TO: StateContract + 'static,
{
    state: TState<FROM>,
    map: for<'r, 's> fn(&'r FROM) -> &'r TO,
    map_mut: for<'r, 's> fn(&'r mut FROM) -> &'r mut TO,
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static> FieldState<FROM, TO> {
    pub fn new<S: Into<TState<FROM>>>(
        state: S,
        map: for<'r, 's> fn(&'r FROM) -> &'r TO,
        map_mut: for<'r, 's> fn(&'r mut FROM) -> &'r mut TO,
    ) -> Self {
        FieldState {
            state: state.into(),
            map,
            map_mut,
        }
    }
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static> State<TO> for FieldState<FROM, TO> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.state.capture_state(env)
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.state.release_state(env)
    }

    fn value(&self) -> ValueRef<TO> {
        let map = self.map;
        ValueRef::map(self.state.value(), |a| { map(a) })
    }

    fn value_mut(&mut self) -> ValueRefMut<TO> {
        let map_mut = self.map_mut;
        ValueRefMut::map(self.state.value_mut(), |a| { map_mut(a) })
    }

    fn set_value(&mut self, value: TO) {
        let map_mut = self.map_mut;
        *ValueRefMut::map(self.state.value_mut(), |a| { map_mut(a) }) = value;
    }
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static> Debug for FieldState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldState")
            .finish()
    }
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static> Into<TState<TO>> for FieldState<FROM, TO> {
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}