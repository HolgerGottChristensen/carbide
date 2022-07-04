use crate::state::{Map1, MapOwnedState, MapWithEnv, RState, StateContract, TState};
use crate::state::readonly::{ReadMap, ReadMapState};
use crate::state::widget_state::Map;

pub trait StateExt<T>: Into<TState<T>> + Clone where T: StateContract {
    /// Example: size.mapped(|t: &f64| { format!("{:.2}", t) })
    fn mapped<TO: StateContract + Default + 'static, M: Map<T, TO> + Clone>(&self, map: M) -> TState<TO> {
        MapOwnedState::<T, TO>::new(self.clone(), move |s: &T, _: &_, _: &_| { map(s) }).into()
    }

    /// This map a state to another state. The resulting state is read-only.
    /// If you need a TState, use [Map1::map()] instead
    ///
    /// Example: size.map(|t: &f64| { format!("{:.2}", t) })
    ///
    /// This will return a RState<String> that will stay updated with the size
    fn map<TO: StateContract>(&self, map: fn(s: &T) -> TO) -> RState<TO> {
        let i: TState<T> = self.clone().into();
        Map1::read_map(i, map)
    }
}

impl<T: StateContract, U> StateExt<T> for U where U: Into<TState<T>> + Clone {}
