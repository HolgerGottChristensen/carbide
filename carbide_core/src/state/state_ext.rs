// use crate::state::{State, StateContract, ValueState, MapOwnedState, Map, TState};
//
// pub trait StateExt<T>: TState<T> + 'static where T: StateContract {
//
// }
//
// impl<T, U> StateExt<T> for U where T: StateContract, U: State<T> + Sized + Clone + 'static {}