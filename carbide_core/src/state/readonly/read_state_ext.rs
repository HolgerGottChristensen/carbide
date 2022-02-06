use carbide_core::state::ReadState;
use crate::state::{MapOwnedState, MapWithEnv, RState, StateContract, TState};
use crate::state::readonly::{ReadMap, ReadMapState, ReadWidgetState};
use crate::state::widget_state::Map;

/*pub trait ReadStateExt<T>: ReadState<T> + Clone where T: StateContract {
    /// Example: size.read_map(|t: &f64| { format!("{:.2}", t) })
    fn read_map<TO: StateContract>(&self, map: fn(&T) -> TO) -> RState<TO> {
        ReadMapState::<T, TO>::new(self.clone(), map)
    }
}

impl<T: StateContract, U> ReadStateExt<T> for U where U: ReadState<T> + Clone {}
*/

impl<T: StateContract> RState<T> {
    pub fn read_map<TO: StateContract, MAP: ReadMap<T, TO>>(&self, map: MAP) -> RState<TO> {
        ReadMapState::<T, TO>::new(self.clone(), map)
    }
}