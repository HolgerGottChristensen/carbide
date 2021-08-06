use crate::state::{State, StateContract, TState};
use crate::state::global_state::GlobalStateContract;
use crate::state::mapped_state::MappedState;
use crate::state::widget_state::WidgetState;

//
// pub trait StateExt<T: StateContract + 'static, GS: GlobalStateContract>: State<T, GS> + Clone + Sized + 'static {
//
// }
//
// impl<X: 'static, T: StateContract + 'static, GS: GlobalStateContract> StateExt<T, GS> for X where X: State<T, GS> + Clone {}
