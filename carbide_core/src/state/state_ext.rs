use crate::state::{MapOwnedState, MapWithEnv, RState, StateContract, TState};
use crate::state::readonly::{ReadMap, ReadMapState};
use crate::state::widget_state::Map;

pub trait StateExt<T>: Into<TState<T>> + Clone where T: StateContract {
    /// Example: size.mapped(|t: &f64| { format!("{:.2}", t) })
    fn mapped<TO: StateContract + Default + 'static, M: Map<T, TO> + Clone>(&self, map: M) -> TState<TO> {
        MapOwnedState::<T, TO>::new(self.clone(), move |s: &T, _: &_, _: &_| { map(s) }).into()
    }

    fn mapped_env<TO: StateContract + Default + 'static, M: MapWithEnv<T, TO>>(&self, map: M) -> TState<TO> {
        MapOwnedState::<T, TO>::new(self.clone(), map).into()
    }

    fn read_map<TO: StateContract, MAP: ReadMap<T, TO>>(&self, map: MAP) -> RState<TO> {
        let state = self.clone().into();
        ReadMapState::<T, TO, MAP>::new(state, map)
    }
}

impl<T: StateContract, U> StateExt<T> for U where U: Into<TState<T>> + Clone {}
