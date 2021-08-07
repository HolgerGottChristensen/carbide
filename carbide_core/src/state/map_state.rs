use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::State;
use crate::state::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct MapState<FROM, TO>
    where FROM: StateContract,
          TO: StateContract {
    state: TState<FROM>,
    map: Box<dyn Map<FROM, TO>>,
    map_mut: Box<dyn MapMut<FROM, TO>>,
}

impl<FROM: StateContract, TO: StateContract> MapState<FROM, TO> {
    pub fn new<S, M1, M2>(state: S, map: M1, map_mut: M2) -> Self
        where S: Into<TState<FROM>>,
              M1: Map<FROM, TO>,
              M2: MapMut<FROM, TO>
    {
        MapState {
            state: state.into(),
            map: Box::new(map),
            map_mut: Box::new(map_mut),
        }
    }
}

impl<FROM: StateContract, TO: StateContract> State<TO> for MapState<FROM, TO> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.state.capture_state(env)
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.state.release_state(env)
    }

    fn value(&self) -> ValueRef<TO> {
        ValueRef::map(self.state.value(), &self.map)
    }

    fn value_mut(&mut self) -> ValueRefMut<TO> {
        ValueRefMut::map(self.state.value_mut(), &self.map_mut)
    }
}

impl<FROM: StateContract, TO: StateContract> Debug for MapState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static> Into<TState<TO>> for MapState<FROM, TO> {
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}

pub trait Map<FROM: StateContract, TO: StateContract>: Fn(&FROM) -> &TO + DynClone + 'static {}

pub trait MapMut<FROM: StateContract, TO: StateContract>: Fn(&mut FROM) -> &mut TO + DynClone + 'static {}

impl<T, FROM: StateContract, TO: StateContract> Map<FROM, TO> for T where T: Fn(&FROM) -> &TO + DynClone + 'static {}

impl<T, FROM: StateContract, TO: StateContract> MapMut<FROM, TO> for T where T: Fn(&mut FROM) -> &mut TO + DynClone + 'static {}

dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> Map<FROM, TO>);
dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> MapMut<FROM, TO>);