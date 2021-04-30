pub use tuple_state::*;

use crate::Color;
use crate::focus::Focus;
pub use crate::state::state::State;

pub mod state;
pub mod environment;
pub mod state_sync;
pub mod global_state;
pub mod mapped_state;
pub mod environment_variable;
pub mod environment_color;
pub mod environment_font_size;
mod environment_state;
pub mod state_key;
pub(crate) mod tuple_state;
pub mod vec_state;

pub type ColorState<GS> = Box<dyn State<Color, GS>>;
pub type StringState<GS> = Box<dyn State<String, GS>>;
pub type U32State<GS> = Box<dyn State<u32, GS>>;
pub type BoolState<GS> = Box<dyn State<bool, GS>>;
pub type F64State<GS> = Box<dyn State<f64, GS>>;
pub type FocusState<GS> = Box<dyn State<Focus, GS>>;
pub type TState<T, GS> = Box<dyn State<T, GS>>;
