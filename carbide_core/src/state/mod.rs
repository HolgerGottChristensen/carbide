use std::fmt::Debug;
use std::rc::Rc;

//pub use self::readonly::ReadStateExt;
pub use flatten::Flatten;
pub use r#impl::*;
pub use tuple_state::*;
pub use util::value_cell::{ValueCell, ValueRef, ValueRefMut};

pub use self::animated_state::*;
pub use self::cache_state::CacheRState;
pub use self::cache_state::CacheTState;
pub use self::env_state::EnvState;
//pub use self::async_state::*;
pub use self::field_state::*;
pub use self::global_state::GlobalState;
pub use self::ignore_writes_state::IgnoreWritesState;
pub use self::index_state::IndexableState;
pub use self::index_state::IndexState;
pub use self::local_state::LocalState;
pub use self::map_owned_state::*;
pub use self::read_state::ReadState;
pub use self::read_widget_state::ReadWidgetState;
pub use self::state::State;
pub use self::state_ext::*;
pub use self::state_key::StateKey;
pub use self::state_sync::NewStateSync;
pub use self::state_sync::StateSync;
pub use self::value_state::ValueState;
pub use self::widget_state::WidgetState;

mod animated_state;
mod local_state;
mod map_owned_state;
mod state;
mod state_ext;
mod state_key;
mod state_sync;
mod value_state;
mod index_state;
mod widget_state;
//mod async_state;
mod field_state;
mod env_state;
mod flatten;
mod ignore_writes_state;
mod r#impl;
mod read_state;
mod read_widget_state;
mod tuple_state;
mod util;
mod cache_state;
mod global_state;

pub type InnerState<T> = Rc<ValueCell<T>>;


pub type TState<T> = WidgetState<T>;
pub type RState<T> = ReadWidgetState<T>;

pub trait StateContract: Clone + Debug + 'static {}

impl<T> StateContract for T where T: Clone + Debug + 'static {}
