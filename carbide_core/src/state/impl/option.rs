use carbide_core::prelude::RState;
use crate::state::{Map1, Map2, StateContract, TState};
use crate::state::ReadWidgetState;

impl<T: StateContract> TState<Option<T>> {
    /// Allows calling is_some in the option. Returns a read-only boolean state. The reason it is
    /// read-only is because it would not be meaningful to set the state with a boolean and expect
    /// the original state to be changed.
    /// Opposite of: [Self::is_none()]
    pub fn is_some(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Option<T>| {
            t.is_some()
        })
    }

    /// This returns a read-only state of boolean with value true if None and false if some.
    /// Related: [Self::is_some()]
    pub fn is_none(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Option<T>| {
            t.is_none()
        })
    }

    /// Returns a boolean state of true if the value within the x state is contained within self.
    pub fn contains<U: StateContract + PartialEq<T>>(&self, x: impl Into<RState<U>>) -> RState<bool> {
        Map2::read_map(self.clone(), x.into(), |option: &Option<T>, value: &U| {
            // Change to https://github.com/rust-lang/rust/issues/62358 when stabilized
            match option {
                Some(y) => value == y,
                None => false,
            }
        })
    }

    /// Will return a state of result that is Ok with the value if the original is Some and
    /// Err with the err state if the original is None.
    ///
    /// When setting the resulting state with Ok it will set the original state to Some with the
    /// value. When setting the resulting state with Err you will set the error state and the
    /// original state will be None.
    pub fn ok_or<E: StateContract>(&self, err: impl Into<TState<E>>) -> TState<Result<T, E>> {
        Map2::map(self.clone(), err.into(), |one: &Option<T>, other: &E| {
            one.clone().ok_or_else(|| other.clone())
        }, |new: Result<T, E>, one: &Option<T>, other: &E| {
            match new {
                Ok(a) => {
                    (Some(Some(a)), None)
                }
                Err(e) => {
                    (Some(None), Some(e))
                }
            }
        })
    }

    /// Return the value inside self if self contains Some. Otherwise return the value
    /// provided as the default. Setting the returned state will make the original state
    /// Some(new value).
    pub fn unwrap_or(&self, default: impl Into<TState<T>>) -> TState<T> {
        Map2::map(self.clone(), default.into(),
            |state: &Option<T>, default: &T| {
                state.clone().unwrap_or(default.clone())
            }, |new, _, _| {
                (Some(Some(new)), None)
            }
        )
    }

    /// Zip the state and the other state to a tuple
    pub fn zip<U: StateContract>(&self, other: impl Into<TState<Option<U>>>) -> TState<Option<(T, U)>> {
        Map2::map(self.clone(), other.into(), |some: &Option<T>, other: &Option<U>| {
            some.clone().zip(other.clone())
        }, |new, some_old, other_old| {
            match new {
                None => {
                    (Some(None), Some(None))
                }
                Some((a, b)) => {
                    (Some(Some(a)), Some(Some(b)))
                }
            }

        })
    }
}

impl<T: StateContract> RState<Option<T>> {
    /// Allows calling is_some in the option. Returns a read-only boolean state. The reason it is
    /// read-only is because it would not be meaningful to set the state with a boolean and expect
    /// the original state to be changed.
    /// Opposite of: [Self::is_none()]
    pub fn is_some(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Option<T>| {
            t.is_some()
        })
    }

    /// This returns a read-only state of boolean with value true if None and false if some.
    /// Related: [Self::is_some()]
    pub fn is_none(&self) -> RState<bool> {
        Map1::read_map(self.clone(), |t: &Option<T>| {
            t.is_none()
        })
    }

    /// Returns a boolean state of true if the value within the x state is contained within self.
    pub fn contains<U: StateContract + PartialEq<T>>(&self, x: impl Into<RState<U>>) -> RState<bool> {
        Map2::read_map(self.clone(), x.into(), |option: &Option<T>, value: &U| {
            // Change to https://github.com/rust-lang/rust/issues/62358 when stabilized
            match option {
                Some(y) => value == y,
                None => false,
            }
        })
    }
}

impl<T: StateContract, U: StateContract> TState<Option<(T, U)>> {

    /// Turns a state of option with a tuple into a tuple of states of options.
    /// There is no meaning full way to set one value without the other, so the
    /// returned states are read only.
    pub fn unzip(&self) -> (RState<Option<T>>, RState<Option<U>>) {
        let a = Map1::read_map(self.clone(), |a: &Option<(T, U)>| {
                match a {
                    None => None,
                    Some((b, _)) => Some(b.clone())
                }
            });
        let b = Map1::read_map(self.clone(), |a: &Option<(T, U)>| {
                match a {
                    None => None,
                    Some((_, b)) => Some(b.clone())
                }
            });

        (a, b)
    }
}

impl<T: StateContract + Default + 'static> TState<Option<T>> {

    /// If the value is None, the default value will be returned.
    /// If you set the value in the returned state, the original state will be modified to a
    /// Some(new value).
    pub fn unwrap_or_default(&self) -> TState<T> {
        Map1::map(
            self.clone(),
            |val: &Option<T>| {
                val.clone().unwrap_or_default()
            }, |new, _| {
                Some(Some(new))
            }
        )
    }
}

impl<T: StateContract + Default> RState<Option<T>> {
    pub fn unwrap_or_default(&self) -> RState<T> {
        Map1::read_map(self.clone(),|t: &Option<T>| { t.clone().unwrap_or_default() } )
    }
}