use std::fmt::Debug;
use std::ops::DerefMut;

use dyn_clone::DynClone;

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{InnerState, State, StringState, ValueCell};
use crate::state::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct MapOwnedState<FROM, TO>
    where
        FROM: StateContract,
        TO: StateContract,
{
    state: TState<FROM>,
    map: Box<dyn Map<FROM, TO>>,
    value: InnerState<TO>,
}

impl<FROM: StateContract, TO: StateContract + Default> MapOwnedState<FROM, TO> {
    pub fn new<M1: Into<TState<FROM>>, M2: Map<FROM, TO>>(state: M1, map: M2) -> Self {
        MapOwnedState {
            state: state.into(),
            map: Box::new(map),
            value: InnerState::new(ValueCell::new(TO::default())),
        }
    }
}

impl<FROM: StateContract, TO: StateContract> MapOwnedState<FROM, TO> {
    pub fn new_with_default<M1: Into<TState<FROM>>, M2: Map<FROM, TO>>(state: M1, map: M2, default: TO) -> Self {
        MapOwnedState {
            state: state.into(),
            map: Box::new(map),
            value: InnerState::new(ValueCell::new(default)),
        }
    }
}

impl<FROM: StateContract, TO: StateContract> State<TO> for MapOwnedState<FROM, TO> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.state.capture_state(env)
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.state.release_state(env)
    }

    fn value(&self) -> ValueRef<TO> {
        let value: TO = (&self.map)(&*self.state.value());
        if let Ok(mut borrow) = self.value.try_borrow_mut() {
            *borrow.deref_mut() = value;
        }
        self.value.borrow()
    }

    fn value_mut(&mut self) -> ValueRefMut<TO> {
        let value: TO = (&self.map)(&*self.state.value());
        if let Ok(mut borrow) = self.value.try_borrow_mut() {
            *borrow.deref_mut() = value;
        }
        self.value.borrow_mut()
    }
}

impl<FROM: StateContract, TO: StateContract> Debug for MapOwnedState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapStateOwned")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static> Into<TState<TO>>
for MapOwnedState<FROM, TO>
{
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}

pub trait Map<FROM: StateContract, TO: StateContract>:
Fn(&FROM) -> TO + DynClone + 'static
{}

impl<T, FROM: StateContract, TO: StateContract> Map<FROM, TO> for T where
    T: Fn(&FROM) -> TO + DynClone + 'static
{}

dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> Map<FROM, TO>);

macro_rules! impl_string_state {
    ($($typ: ty),*) => {
        $(
            impl Into<StringState> for TState<$typ> {
                fn into(self) -> StringState {
                    MapOwnedState::new(self, |s: &$typ| {s.to_string()}).into()
                }
            }
        )*

    };
}

impl_string_state!(
    i8, u8, i16, u16,
    i32, u32, i64, u64,
    i128, u128, f32, f64,
    bool, char, isize, usize
);