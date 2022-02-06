use carbide_core::state::RState;
use crate::state::readonly::{ReadWidgetState};
use crate::state::{ReadState, StateContract, ValueState, WidgetState};

impl<T: StateContract, E: StateContract> ReadWidgetState<Result<T, E>> {
    pub fn is_ok(&self) -> RState<bool> {
        self.read_map(|t: &Result<T, E>| { t.is_ok() })
    }

    pub fn is_err(&self) -> RState<bool> {
        self.read_map(|t: &Result<T, E>| { t.is_err() })
    }
}

impl<T: StateContract> ReadWidgetState<Option<T>> {
    pub fn is_some(&self) -> RState<bool> {
        self.read_map(|t: &Option<T>| { t.is_some() })
    }

    pub fn is_none(&self) -> RState<bool> {
        self.read_map(|t: &Option<T>| { t.is_none() })
    }
}

impl<T: StateContract + Default + 'static> ReadWidgetState<Option<T>> {
    pub fn unwrap_or_default(&self) -> RState<T> {
        self.read_map(|t: &Option<T>| { t.clone().unwrap_or_default() })
    }
}

impl<T: StateContract> From<T> for RState<T> {
    fn from(t: T) -> Self {
        ReadWidgetState::ReadWriteState(ValueState::new(t))
    }
}

impl From<u32> for RState<f64> {
    fn from(t: u32) -> Self {
        ValueState::new(t as f64).into()
    }
}

impl From<&str> for RState<String> {
    fn from(t: &str) -> Self {
        ValueState::new(t.to_string()).into()
    }
}