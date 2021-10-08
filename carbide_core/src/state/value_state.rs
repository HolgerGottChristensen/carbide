use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::environment::Environment;
use crate::state::{BoolState, MapOwnedState, MapState, ResStringState, State, StateContract, StringState, TState};
use crate::state::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct ValueState<T>
    where
        T: StateContract,
{
    value: T,
}

impl<T: StateContract> ValueState<T> {
    pub fn new(value: T) -> Box<Self> {
        Box::new(ValueState { value })
    }
}

impl<T: StateContract> Deref for ValueState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: StateContract> DerefMut for ValueState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: StateContract> State<T> for ValueState<T> {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Borrow(&mut self.value)
    }
}

impl<T: StateContract> Debug for ValueState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::ValueState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract + 'static> Into<TState<T>> for Box<ValueState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}

/// This should implement into T state for pretty much all T.
impl<T: StateContract + 'static> From<T> for TState<T> {
    fn from(t: T) -> Self {
        WidgetState::new(ValueState::new(t))
    }
}

impl From<u32> for TState<f64> {
    fn from(t: u32) -> Self {
        WidgetState::new(ValueState::new(t as f64))
    }
}

impl From<&str> for TState<String> {
    fn from(t: &str) -> Self {
        WidgetState::new(ValueState::new(t.to_string()))
    }
}

impl<T: StateContract + Default + 'static> Into<TState<Result<T, String>>> for TState<T> {
    fn into(self) -> TState<Result<T, String>> {
        MapOwnedState::new_with_default(self, |val: &T, env: &Environment| {
            Ok(val.clone())
        }, Ok(T::default())).into()
    }
}

impl Into<ResStringState> for TState<Result<u32, String>> {
    fn into(self) -> ResStringState {
        MapOwnedState::new_with_default(self, |value: &Result<u32, String>, env: &Environment| {
            match value {
                Ok(val) => { Ok(val.to_string()) }
                Err(val) => { Err(val.to_string()) }
            }
        }, Ok("".to_string())).into()
    }
}

impl Into<StringState> for ResStringState {
    fn into(self) -> StringState {
        MapState::new(self, (), |res: &Result<String, String>, val| {
            match res.as_ref() {
                Ok(a) | Err(a) => {
                    a
                }
            }
        }, |res: &mut Result<String, String>, val| {
            match res.as_mut() {
                Ok(a) | Err(a) => {
                    a
                }
            }
        }).into()
    }
}

impl Into<BoolState> for ResStringState {
    fn into(self) -> BoolState {
        self.mapped(|val: &Result<String, String>| {
            val.is_err()
        })
    }
}
