use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::environment::Environment;
use crate::state::{BoolState, MapOwnedState, MapState, ResStringState, State, StateContract, StateExt, StringState, TState};
use crate::state::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct ValueState<T>
    where
        T: StateContract,
{
    value: T,
}

impl<T: StateContract + 'static> ValueState<T> {
    pub fn new(value: T) -> TState<T> {
        Box::new(ValueState { value }).into()
    }

    pub fn new_raw(value: T) -> Box<Self> {
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
    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: T) {
        self.value = value;
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
        ValueState::new(t)
    }
}

impl From<u32> for TState<f64> {
    fn from(t: u32) -> Self {
        ValueState::new(t as f64)
    }
}

impl From<&str> for TState<String> {
    fn from(t: &str) -> Self {
        ValueState::new(t.to_string())
    }
}

impl<T: StateContract + Default + 'static> Into<TState<Result<T, String>>> for TState<T> {
    fn into(self) -> TState<Result<T, String>> {
        MapOwnedState::new_with_default(self, |val: &T, _: &_, env: &Environment| {
            Ok(val.clone())
        }, Ok(T::default())).into()
    }
}

macro_rules! impl_res_state_plain {
    ($($typ: ty),*) => {
        $(
        impl Into<ResStringState> for TState<Result<$typ, String>> {
            fn into(self) -> ResStringState {
                MapOwnedState::new_with_default_and_rev(self, |value: &Result<$typ, String>, _: &_, env: &Environment| {
                    match value {
                        Ok(val) => { Ok(val.to_string()) }
                        Err(val) => { Err(val.to_string()) }
                    }
                }, |val: &Result<String, String>| {
                    match val {
                        Ok(s) | Err(s) => {
                            <$typ>::from_str(s)
                                .map_err(|_| s.to_string())
                        }
                    }
                },Ok("".to_string())).into()
            }
        }
        )*
    }
}

impl_res_state_plain! {
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize
}

impl Into<ResStringState> for TState<Result<f32, String>> {
    fn into(self) -> ResStringState {
        MapOwnedState::new_with_default_and_rev(self, |value: &Result<f32, String>, prev: &Result<String, String>, env: &Environment| {
            match (value, prev) {
                (Ok(val), Ok(a)) => {
                    if let Ok(v) = f32::from_str(a) {
                        if *val == v {
                            return Ok(a.clone())
                        }
                    }
                    Ok(val.to_string())
                }
                (Ok(val), _) => Ok(val.to_string()),
                (Err(val), _) => Err(val.to_string()),
            }
        }, |val: &Result<String, String>| {
            match val {
                Ok(s) | Err(s) => {
                    f32::from_str(s)
                        .map_err(|_| s.to_string())
                }
            }
        }, Ok("".to_string())).into()
    }
}

impl Into<ResStringState> for TState<Result<f64, String>> {
    fn into(self) -> ResStringState {
        MapOwnedState::new_with_default_and_rev(self, |value: &Result<f64, String>, prev: &Result<String, String>, env: &Environment| {
            match (value, prev) {
                (Ok(val), Ok(a)) => {
                    if let Ok(v) = f64::from_str(a) {
                        if *val == v {
                            return Ok(a.clone())
                        }
                    }
                    Ok(val.to_string())
                }
                (Ok(val), _) => Ok(val.to_string()),
                (Err(val), _) => Err(val.to_string()),
            }
        }, |val: &Result<String, String>| {
            match val {
                Ok(s) | Err(s) => {
                    f64::from_str(s)
                        .map_err(|_| s.to_string())
                }
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
        }, |res: &String| {
            Ok(res.to_string())
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
