use std::fmt::Debug;
use std::rc::Rc;

pub use r#impl::*;
pub use tuple_state::*;
pub use util::value_cell::{ValueCell, ValueRef, ValueRefMut};

pub use self::animated_state::*;
pub use self::cache_state::CachedReadState;
pub use self::cache_state::CachedState;
//pub use self::async_state::*;
pub use self::field_state::*;
pub use self::global_state::GlobalState;
pub use self::ignore_writes_state::IgnoreWritesState;
pub use self::index_state::IndexState;
pub use self::local_state::LocalState;
pub use self::read_state::*;
pub use self::into_read_state::*;
pub use self::into_state::*;
pub use self::state::*;
pub use self::state_ext::*;
pub use self::state_sync::StateSync;
pub use self::value_state::ValueState;
pub use self::logging_state::*;
pub use self::transition_state::*;
pub use self::static_state::*;
pub use self::functor::*;

mod animated_state;
mod local_state;
mod state;
mod state_ext;
mod state_sync;
mod value_state;
mod index_state;
//mod async_state;
mod field_state;
mod ignore_writes_state;
mod r#impl;
mod read_state;
mod tuple_state;
mod util;
mod cache_state;
mod global_state;
mod into_state;
mod into_read_state;
mod transition_state;
mod logging_state;
mod static_state;
mod functor;
mod flatten;
mod empty_state;
mod extensions;
pub use carbide_derive::StateValue;

pub type InnerState<T> = Rc<ValueCell<T>>;

pub trait StateContract: Clone + Debug + 'static {}

impl<T> StateContract for T where T: Clone + Debug + 'static {}

#[cfg(test)]
mod tests {
    use carbide_core::state::Map1;
    use crate::state::{GlobalState, LocalState, ReadState, State};

    #[test]
    fn mutate_mapped_local_state() {
        let state = LocalState::new(0);

        let mut mapped = Map1::map(state.clone(), |val| {
            *val + 1
        }, |new, mut val| {
            *val = new - 1;
        });

        assert_eq!(*state.value() + 1, *mapped.value());

        *mapped.value_mut() += 1;

        assert_eq!(*state.value() + 1, *mapped.value());

        println!("State: {}, Mapped: {}", state.value(), mapped.value());
    }

    #[test]
    fn mutate_mapped_global_state() {
        let state = GlobalState::new(0);

        let mut mapped = Map1::map(state.clone(), |val| {
            *val + 1
        }, |new, mut val| {
            *val = new - 1;
        });

        assert_eq!(*state.value() + 1, *mapped.value());

        *mapped.value_mut() += 1;

        assert_eq!(*state.value() + 1, *mapped.value());

        println!("State: {}, Mapped: {}", state.value(), mapped.value());
    }

    #[test]
    fn mutate_mapped_mapped_local_state() {
        let state = LocalState::new(0);

        let mut mapped = Map1::map(state.clone(), |val| {
            *val + 1
        }, |new, mut val| {
            *val = new - 1;
        });

        let mapped2 = Map1::read_map(mapped.clone(), |val| {
            *val * 2
        });

        assert_eq!(*state.value() + 1, *mapped.value());

        *mapped.value_mut() += 1;

        assert_eq!(*state.value() + 1, *mapped.value());

        println!("State: {}, Mapped: {}, Mapped2: {}", state.value(), mapped.value(), mapped2.value());
    }
}