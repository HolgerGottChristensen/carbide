use crate::state::{State, StateContract};
use crate::state::global_state::GlobalState;
use crate::state::mapped_state::MappedState;

pub trait StateExt<T: StateContract + 'static, GS: GlobalState>: State<T, GS> + Sized + 'static {
    fn mapped<U: StateContract + 'static>(self, map: fn(&T) -> U) -> Box<dyn State<U, GS>> {
        let latest_value = self.get_latest_value().clone();
        MappedState::new(Box::new(self), map, map(&latest_value))
    }
}

impl<X: 'static, T: StateContract + 'static, GS: GlobalState> StateExt<T, GS> for X where X: State<T, GS> {}
