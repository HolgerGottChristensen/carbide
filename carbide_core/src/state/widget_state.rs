use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::prelude::Environment;
use crate::prelude::value_cell::{ValueRef, ValueRefMut};
use crate::state::StateContract;
use crate::state::global_state::GlobalStateContainer;
pub use crate::state::State;
use crate::widget::GlobalStateContract;

pub struct WidgetState<T>(Box<dyn State<T>>);

impl<T: StateContract> WidgetState<T> {
    pub fn new(item: Box<dyn State<T>>) -> WidgetState<T> {
        WidgetState(item)
    }
}

/*impl<T: StateContract + 'static, GS: GlobalStateContract> WidgetState<T, GS> {
    pub fn mapped<U: StateContract + Default + 'static>(self, map: fn(&Self) -> U) -> WidgetState<U, GS> {
        WidgetState::new(MappedState::new(self, map))
    }
}*/

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