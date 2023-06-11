use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use carbide_core::state::Map1;
use carbide_core::state::NewStateSync;

use crate::Color;
use crate::environment::Environment;
use crate::render::Style;
use crate::state::{IntoReadState, ReadWidgetState, RState, State, StateContract, StateExt, TState};
use crate::state::{ValueRef, ValueRefMut};
use crate::state::ReadState;
use crate::state::widget_state::WidgetState;
use crate::widget::{Gradient};

/// # ValueState
/// Value state is a state that can be used for constants and values that are not shared. When
/// cloning this state the value is cloned and when the clone changes the original will not
/// change. For shared state use [LocalState].
///
/// One important thing to know is that state might be cloned depending on the widgets you use.
/// When storing state inside a ValueState it is important to remember, because if you store
/// large values those will be cloned as well. Using a local state, it is only a Rc that will be
/// cloned which will be way more efficient and use much less space.
///
/// Local state implements [NewStateSync] where [NewStateSync::sync()] is a NoOp.
#[derive(Clone)]
pub struct ValueState<T>
where
    T: StateContract,
{
    /// The value contained as the state
    value: T,
}

impl<T: StateContract> ValueState<T> {
    pub fn new(value: T) -> TState<T> {
        WidgetState::Value(ValueState { value })
    }

    pub fn new_raw(value: T) -> Box<Self> {
        Box::new(ValueState { value })
    }
}

impl<T: StateContract> NewStateSync for ValueState<T> {}

impl<T: StateContract> ReadState for ValueState<T> {
    type T = T;
    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }
}

impl<T: StateContract> State for ValueState<T> {
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

impl From<&str> for RState<String> {
    fn from(t: &str) -> Self {
        ReadWidgetState::ReadWriteState(ValueState::new(t.to_string()))
    }
}

// impl<T: StateContract + Default + 'static> Into<TState<Result<T, String>>> for TState<T> {
//     fn into(self) -> TState<Result<T, String>> {
//         MapOwnedState::new_with_default_and_rev(
//             self,
//             |val: &T, _: &_, _: &Environment| Ok(val.clone()),
//             |val: &Result<T, String>| val.as_ref().ok().map(|a| a.clone()),
//             Ok(T::default()),
//         )
//         .into()
//     }
// }

macro_rules! impl_res_state_plain {
    ($($typ: ty),*) => {
        $(
        impl Into<TState<Result<String, String>>> for TState<Result<$typ, String>> {
            fn into(self) -> TState<Result<String, String>> {
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

// impl_res_state_plain! {
//     u8, u16, u32, u64, u128, usize,
//     i8, i16, i32, i64, i128, isize
// }
//
// impl Into<TState<Result<String, String>>> for TState<Result<f32, String>> {
//     fn into(self) -> TState<Result<String, String>> {
//         MapOwnedState::new_with_default_and_rev(
//             self,
//             |value: &Result<f32, String>, prev: &Result<String, String>, _: &Environment| match (
//                 value, prev,
//             ) {
//                 (Ok(val), Ok(a)) => {
//                     if let Ok(v) = f32::from_str(a) {
//                         if *val == v {
//                             return Ok(a.clone());
//                         }
//                     }
//                     Ok(val.to_string())
//                 }
//                 (Ok(val), _) => Ok(val.to_string()),
//                 (Err(val), _) => Err(val.to_string()),
//             },
//             |val: &Result<String, String>| match val {
//                 Ok(s) | Err(s) => Some(f32::from_str(s).map_err(|_| s.to_string())),
//             },
//             Ok("".to_string()),
//         )
//         .into()
//     }
// }
//
// impl Into<TState<Result<String, String>>> for TState<Result<f64, String>> {
//     fn into(self) -> TState<Result<String, String>> {
//         MapOwnedState::new_with_default_and_rev(
//             self,
//             |value: &Result<f64, String>, prev: &Result<String, String>, _: &Environment| match (
//                 value, prev,
//             ) {
//                 (Ok(val), Ok(a)) => {
//                     if let Ok(v) = f64::from_str(a) {
//                         if *val == v {
//                             return Ok(a.clone());
//                         }
//                     }
//                     Ok(val.to_string())
//                 }
//                 (Ok(val), _) => Ok(val.to_string()),
//                 (Err(val), _) => Err(val.to_string()),
//             },
//             |val: &Result<String, String>| match val {
//                 Ok(s) | Err(s) => Some(f64::from_str(s).map_err(|_| s.to_string())),
//             },
//             Ok("".to_string()),
//         )
//         .into()
//     }
// }

impl Into<TState<String>> for TState<Result<String, String>> {
    fn into(self) -> TState<String> {
        Map1::map_cached(
            self,
            |res: &Result<String, String>| match res.as_ref() {
                Ok(s) | Err(s) => s.clone(),
            },
            |new, _| Some(Ok(new)),
        )
    }
}

// impl From<TState<Result<String, String>>> for RState<String> {
//     fn from(t: TState<Result<String, String>>) -> RState<String> {
//         Map1::read_map_cached(
//             t,
//             |res: &Result<String, String>| match res.as_ref() {
//                 Ok(s) | Err(s) => s.clone(),
//             }
//         )
//     }
// }

/*impl Into<BoolState> for ResStringState {
    fn into(self) -> BoolState {
        self.mapped(|val: &Result<String, String>| val.is_err())
    }
}*/

// impl Into<RState<Style>> for TState<Color> {
//     fn into(self) -> RState<Style> {
//         self.map(|c: &Color| Style::from(*c))
//     }
// }

// impl Into<TState<Style>> for TState<Color> {
//     fn into(self) -> TState<Style> {
//         self.map(|c: &Color| Style::from(*c))
//             .ignore_writes()
//     }
// }

impl Into<TState<Style>> for Color {
    fn into(self) -> TState<Style> {
        ValueState::new(Style::Color(self))
    }
}
