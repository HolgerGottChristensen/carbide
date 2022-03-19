use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use carbide_core::prelude::NewStateSync;
use crate::Color;

use crate::environment::Environment;
use crate::prelude::{AdvancedColor, ReadState};
use crate::state::{BoolState, MapOwnedState, ResStringState, State, StateContract, StateExt, StringState, TState, RState, MapState};
use crate::state::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;
use crate::widget::Gradient;

/// # ValueState
/// Value state is a state that can be used for constants and values that are not shared. When
/// cloning this state the value is cloned, but when the clone changes the original will not
/// change. For shared state use [LocalState].
///
/// ValueState is [Listenable] which means you can subscribe to it changing. When listening to
/// state changes and cloning the state, it will listen to all changed also from the clone.
/// This is not finally decided to be the correct behavior so dont build code that depend upon this.
///
/// Local state implements [NewStateSync] but without implementing any behavior when
/// [NewStateSync::sync()] is called.
#[derive(Clone)]
pub struct ValueState<T> where T: StateContract {
    /// The value contained as the state
    value: T,
}

impl<T: StateContract> ValueState<T> {
    pub fn new(value: T) -> TState<T> {
        Self::new_raw(value).into()
    }

    pub fn new_raw(value: T) -> Box<Self> {
        Box::new(ValueState {
            value,
        })
    }
}

impl<T: StateContract> Deref for ValueState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: StateContract> DerefMut for ValueState<T> {
    /// You should make sure to call notify manually after modifying the state using deref_mut.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: StateContract> NewStateSync for ValueState<T> {}

impl<T: StateContract> ReadState<T> for ValueState<T> {
    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }
}

impl<T: StateContract> State<T> for ValueState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: T) {
        self.value = value;
    }

}

impl<T: StateContract> Debug for ValueState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract> Into<TState<T>> for Box<ValueState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}

// This should implement into T state for pretty much all T.
impl<T: StateContract> From<T> for TState<T> {
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
        MapOwnedState::new_with_default_and_rev(self, |val: &T, _: &_, _: &Environment| {
            Ok(val.clone())
        }, |val: &Result<T, String>| {
            val.as_ref().ok().map(|a| a.clone())
        }, Ok(T::default())).into()
    }
}

macro_rules! impl_res_state_plain {
    ($($typ: ty),*) => {
        $(
        impl Into<ResStringState> for TState<Result<$typ, String>> {
            fn into(self) -> ResStringState {
                MapOwnedState::new_with_default_and_rev(self, |value: &Result<$typ, String>, _: &_, _: &Environment| {
                    match value {
                        Ok(val) => { Ok(val.to_string()) }
                        Err(val) => { Err(val.to_string()) }
                    }
                }, |val: &Result<String, String>| {
                    match val {
                        Ok(s) | Err(s) => {
                            Some(<$typ>::from_str(s)
                                .map_err(|_| s.to_string()))
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
        MapOwnedState::new_with_default_and_rev(self, |value: &Result<f32, String>, prev: &Result<String, String>, _: &Environment| {
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
                    Some(f32::from_str(s)
                        .map_err(|_| s.to_string()))
                }
            }
        }, Ok("".to_string())).into()
    }
}

impl Into<ResStringState> for TState<Result<f64, String>> {
    fn into(self) -> ResStringState {
        MapOwnedState::new_with_default_and_rev(self, |value: &Result<f64, String>, prev: &Result<String, String>, _: &Environment| {
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
                    Some(f64::from_str(s)
                        .map_err(|_| s.to_string()))
                }
            }
        }, Ok("".to_string())).into()
    }
}

impl Into<StringState> for ResStringState {
    fn into(self) -> StringState {
        MapState::new(self, (), |res: &Result<String, String>, _| {
            match res.as_ref() {
                Ok(a) | Err(a) => {
                    a
                }
            }
        }, |res: &mut Result<String, String>, _| {
            match res.as_mut() {
                Ok(a) | Err(a) => {
                    a
                }
            }
        }, |res: &String| {
            Some(Ok(res.to_string()))
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

impl Into<RState<AdvancedColor>> for TState<Color> {
    fn into(self) -> RState<AdvancedColor> {
        self.read_map(|c: &Color| { AdvancedColor::from(*c) })
    }
}

impl Into<TState<AdvancedColor>> for TState<Color> {
    fn into(self) -> TState<AdvancedColor> {
        self.read_map(|c: &Color| { AdvancedColor::from(*c) }).ignore_writes()
    }
}

impl Into<TState<AdvancedColor>> for Color {
    fn into(self) -> TState<AdvancedColor> {
        ValueState::new(AdvancedColor::Color(self))
    }
}

impl Into<TState<AdvancedColor>> for Gradient {
    fn into(self) -> TState<AdvancedColor> {
        ValueState::new(AdvancedColor::SingleGradient(self))
    }
}
