use crate::state::{Map1, Map2, State, StateContract, TState};
use carbide_core::prelude::RState;

impl<T: StateContract, E: StateContract> TState<Result<T, E>> {
    /// Return a boolean state that is true if the original state is Ok.
    pub fn is_ok(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Result<T, E>| t.is_ok())
    }

    /// Return a boolean state that is true if the original state is Err.
    pub fn is_err(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Result<T, E>| t.is_err())
    }

    /// Returns a boolean state of true if the value within the x state is contained within self.
    pub fn contains<U: StateContract + PartialEq<T>>(
        &self,
        x: impl Into<RState<U>>,
    ) -> RState<bool> {
        Map2::read_map(self.clone(), x.into(), |res: &Result<T, E>, value: &U| {
            // Change to https://github.com/rust-lang/rust/issues/62358 when stabilized
            match res {
                Ok(y) => value == y,
                Err(_) => false,
            }
        })
    }

    /// Return the value inside self if self contains Ok. Otherwise return the value
    /// provided as the default. Setting the returned state will make the original state
    /// Ok(new value).
    pub fn unwrap_or(self, default: impl Into<TState<T>>) -> TState<T> {
        Map2::map(
            self.clone(),
            default.into(),
            |state: &Result<T, E>, default: &T| state.clone().unwrap_or(default.clone()),
            |new, _, _| (Some(Ok(new)), None),
        )
    }
}

impl<T: StateContract, E: StateContract> RState<Result<T, E>> {
    pub fn is_ok(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Result<T, E>| t.is_ok())
    }

    pub fn is_err(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Result<T, E>| t.is_err())
    }
}

impl<T: StateContract + Default + 'static, E: StateContract> TState<Result<T, E>> {
    /// If the value is Err, the default value will be returned.
    /// If you set the value in the returned state, the original state will be modified to a
    /// Ok(new value).
    pub fn unwrap_or_default(&self) -> TState<T> {
        Map1::map(
            self.clone(),
            |val: &Result<T, E>| val.clone().unwrap_or_default(),
            |new, _| Some(Ok(new)),
        )
    }
}
