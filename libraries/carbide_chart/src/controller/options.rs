//! Controller options, each a value or a state of one, as every option of a chart is: a builder
//! takes either, and the controller reads it as last synced.

use carbide::state::{AnyReadState, IntoReadState, ReadStateExt, StateContract, ValueState};

/// A controller's option: a value or a state of one.
pub(crate) type Opt<T> = Box<dyn AnyReadState<T=T>>;

/// An option that always holds `value`, which never reports a change when synced.
pub(crate) fn constant<T: StateContract>(value: T) -> Opt<T> {
    Box::new(ValueState::new(value))
}

/// An option from `value`, a value or a state of one.
pub(crate) fn option<T: StateContract>(value: impl IntoReadState<T>) -> Opt<T> {
    value.into_read_state().as_dyn_read()
}

/// The option's value, as last synced.
pub(crate) fn get<T: StateContract + Copy>(option: &Opt<T>) -> T {
    *option.value_dyn()
}
