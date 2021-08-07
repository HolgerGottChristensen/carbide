use std::fmt::Debug;

pub use tuple_state::*;

use crate::Color;
use crate::focus::Focus;
pub use crate::state::state::State;
use crate::state::widget_state::WidgetState;

//pub use self::mapped_state::MappedState;

pub mod state;
pub mod state_sync;
pub mod global_state;
pub mod mapped_state;
pub mod state_key;
pub mod tuple_state;
pub mod vec_state;
pub mod widget_state;
pub mod state_ext;
pub mod local_state;
pub mod value_state;
pub mod env_state;
pub mod value_cell;

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
