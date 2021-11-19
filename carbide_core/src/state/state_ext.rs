use crate::state::{MapOwnedState, MapWithEnv, State, StateContract, TState};
use crate::state::widget_state::Map;

pub trait StateExt<T>: Into<TState<T>> + Clone where T: StateContract + 'static {
    fn mapped<TO: StateContract + Default + 'static, M: Map<T, TO> + Clone>(&self, map: M) -> TState<TO> {
        MapOwnedState::<T, TO>::new(self.clone(), move |s: &T, _: &_, _: &_| { map(s) }).into()
    }

    fn mapped_env<TO: StateContract + Default + 'static, M: MapWithEnv<T, TO>>(&self, map: M) -> TState<TO> {
        MapOwnedState::<T, TO>::new(self.clone(), map).into()
    }
}

impl<T: StateContract + 'static, U> StateExt<T> for U where U: Into<TState<T>> + Clone {}
