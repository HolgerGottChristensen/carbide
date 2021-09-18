use std::fmt;
use std::fmt::{Debug, Formatter};

use dyn_clone::DynClone;

use crate::prelude::Environment;
use crate::state::{Map, MapOwnedState, StateContract, TState};
pub use crate::state::State;
use crate::state::value_cell::{ValueRef, ValueRefMut};

pub struct WidgetState<T>(Box<dyn State<T>>);

impl<T: StateContract> WidgetState<T> {
    pub fn new(item: Box<dyn State<T>>) -> WidgetState<T> {
        WidgetState(item)
    }
}

impl<T: StateContract + 'static> WidgetState<T> {
    pub fn mapped<TO: StateContract + Default + 'static, M: MapNoEnv<T, TO> + Clone>(&self, map: M) -> TState<TO> {
        MapOwnedState::<T, TO>::new(self.clone(), move |s: &T, _: &_| { map(s) }).into()
    }

    pub fn mapped_env<TO: StateContract + Default + 'static, M: Map<T, TO>>(&self, map: M) -> TState<TO> {
        MapOwnedState::<T, TO>::new(self.clone(), map).into()
    }
}

impl<T: StateContract> Debug for WidgetState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: StateContract> Clone for WidgetState<T> {
    fn clone(&self) -> Self {
        WidgetState(self.0.clone())
    }
}

impl<T: StateContract> Into<WidgetState<T>> for Box<dyn State<T>> {
    fn into(self) -> WidgetState<T> {
        WidgetState(self)
    }
}

impl<T: StateContract> State<T> for WidgetState<T> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.0.capture_state(env)
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.0.release_state(env)
    }

    fn value(&self) -> ValueRef<T> {
        self.0.value()
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.0.value_mut()
    }
}

pub trait MapNoEnv<FROM: StateContract, TO: StateContract>:
Fn(&FROM) -> TO + DynClone + 'static
{}

impl<T, FROM: StateContract, TO: StateContract> MapNoEnv<FROM, TO> for T where
    T: Fn(&FROM) -> TO + DynClone + 'static
{}

dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> MapNoEnv<FROM, TO>);