use std::fmt::Debug;

pub use tuple_state::*;

use crate::{Color, DeserializeOwned, Serialize};
use crate::focus::Focus;
pub use crate::state::state::State;
use crate::state::widget_state::WidgetState;

pub use self::mapped_state::MappedState;

pub mod state;
pub mod state_sync;
pub mod global_state;
pub mod mapped_state;
pub mod state_key;
pub mod tuple_state;
pub mod vec_state;
pub mod widget_state;
pub mod state_ext;

pub type ColorState<GS> = TState<Color, GS>;
pub type StringState<GS> = TState<String, GS>;
pub type U32State<GS> = TState<u32, GS>;
pub type UsizeState<GS> = TState<usize, GS>;
pub type BoolState<GS> = TState<bool, GS>;
pub type F64State<GS> = TState<f64, GS>;
pub type FocusState<GS> = TState<Focus, GS>;
pub type TState<T, GS> = WidgetState<T, GS>;

pub trait StateContract: Serialize + Clone + Debug + DeserializeOwned + Default {}

impl<T> StateContract for T where T: Serialize + Clone + Debug + DeserializeOwned + Default {}
