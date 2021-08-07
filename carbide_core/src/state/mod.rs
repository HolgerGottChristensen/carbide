use std::fmt::Debug;
use std::rc::Rc;

use crate::Color;
use crate::focus::Focus;
pub(crate) use crate::state::value_cell::{ValueCell, ValueRef, ValueRefMut};

pub use self::env_state::EnvState;
pub use self::global_state::GlobalState;
pub use self::local_state::LocalState;
pub use self::map_state::{Map, MapMut, MapState};
pub use self::state::State;
pub use self::state_key::StateKey;
pub use self::state_sync::{NoLocalStateSync, StateSync};
pub use self::value_state::ValueState;
pub use self::widget_state::WidgetState;

mod state;
mod state_sync;
mod global_state;
mod map_state;
mod state_key;
mod vec_state;
mod widget_state;
mod local_state;
mod value_state;
mod env_state;
mod value_cell;

pub(crate) type InnerState<T> = Rc<ValueCell<T>>;

pub type ColorState = TState<Color>;
pub type StringState = TState<String>;
pub type U32State = TState<u32>;
pub type I32State = TState<i32>;
pub type UsizeState = TState<usize>;
pub type BoolState = TState<bool>;
pub type F64State = TState<f64>;
pub type FocusState = TState<Focus>;
pub type TState<T> = WidgetState<T>;

pub trait StateContract: Clone + Debug {}

impl<T> StateContract for T where T: Clone + Debug {}
