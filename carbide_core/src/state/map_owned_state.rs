use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;
use carbide_core::prelude::{NewStateSync, Listenable, Listener};

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{InnerState, LocalState, ReadState, State, StringState, ValueCell, ValueState};
use crate::state::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct MapOwnedState<FROM, TO>
    where
        FROM: StateContract,
        TO: StateContract,
{
    state: TState<FROM>,
    map: Box<dyn MapWithEnv<FROM, TO>>,
    map_rev: Option<Box<dyn MapRev<FROM, TO>>>,
    value: InnerState<TO>,
}

impl<FROM: StateContract, TO: StateContract + Default> MapOwnedState<FROM, TO> {
    pub fn new<M1: Into<TState<FROM>>, M2: MapWithEnv<FROM, TO>>(state: M1, map: M2) -> Self {
        MapOwnedState {
            state: state.into(),
            map: Box::new(map),
            map_rev: None,
            value: InnerState::new(ValueCell::new(TO::default())),
        }
    }
}

impl<FROM: StateContract, TO: StateContract> MapOwnedState<FROM, TO> {
    pub fn new_with_default<M1: Into<TState<FROM>>, M2: MapWithEnv<FROM, TO>>(state: M1, map: M2, default: TO) -> Self {
        MapOwnedState {
            state: state.into(),
            map: Box::new(map),
            map_rev: None,
            value: InnerState::new(ValueCell::new(default)),
        }
    }

    pub fn new_with_default_and_rev<I: Into<TState<FROM>>, M1: MapWithEnv<FROM, TO>, M2: MapRev<FROM, TO>>(state: I, map: M1, map_rev: M2, default: TO) -> Self {
        MapOwnedState {
            state: state.into(),
            map: Box::new(map),
            map_rev: Some(Box::new(map_rev)),
            value: InnerState::new(ValueCell::new(default)),
        }
    }
}

impl<FROM: StateContract, TO: StateContract> NewStateSync for MapOwnedState<FROM, TO> {
    fn sync(&mut self, env: &mut Environment) {
        self.state.sync(env);

        if let Ok(mut borrow) = self.value.try_borrow_mut() {
            let value: TO = (&self.map)(&*self.state.value(), borrow.deref(), env);
            *borrow.deref_mut() = value;
        }
    }
}

impl<FROM: StateContract, TO: StateContract> Listenable<TO> for MapOwnedState<FROM, TO> {
    fn subscribe(&self, subscriber: Box<dyn Listener<TO>>) {
        todo!()
    }
}

impl<FROM: StateContract, TO: StateContract> ReadState<TO> for MapOwnedState<FROM, TO> {
    fn value(&self) -> ValueRef<TO> {
        self.value.borrow()
    }
}

impl<FROM: StateContract, TO: StateContract> State<TO> for MapOwnedState<FROM, TO> {
    fn value_mut(&mut self) -> ValueRefMut<TO> {
        self.value.borrow_mut()
    }

    /// Set value will only update its containing state if the map_rev is specified.
    fn set_value(&mut self, value: TO) {
        if let Some(rev_map) = &self.map_rev {
            let from: Option<FROM> = (rev_map)(&value);
            if let Some(from) = from {
                self.state.set_value(from);
            }
        }
    }

    fn notify(&self) {
        todo!()
    }

    fn update_dependent(&mut self) {
        if let Some(rev_map) = &self.map_rev {
            let from: Option<FROM> = (rev_map)(&*self.value.borrow());
            if let Some(from) = from {
                self.state.set_value(from);
            }
        }
    }
}

impl<FROM: StateContract, TO: StateContract> Debug for MapOwnedState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapStateOwned")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract, TO: StateContract> Into<TState<TO>>
for MapOwnedState<FROM, TO>
{
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}

pub trait MapWithEnv<FROM: StateContract, TO: StateContract>:
Fn(&FROM, &TO, &Environment) -> TO + DynClone + 'static
{}

impl<T, FROM: StateContract, TO: StateContract> MapWithEnv<FROM, TO> for T where
    T: Fn(&FROM, &TO, &Environment) -> TO + DynClone + 'static
{}

pub trait MapRev<FROM: StateContract, TO: StateContract>:
Fn(&TO) -> Option<FROM> + DynClone + 'static
{}

impl<T, FROM: StateContract, TO: StateContract> MapRev<FROM, TO> for T where
    T: Fn(&TO) -> Option<FROM> + DynClone + 'static
{}

dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> MapWithEnv<FROM, TO>);

dyn_clone::clone_trait_object!(<FROM: StateContract, TO: StateContract> MapRev<FROM, TO>);

macro_rules! impl_string_state {
    ($($typ: ty),*) => {
        $(
            impl Into<StringState> for TState<$typ> {
                fn into(self) -> StringState {
                    MapOwnedState::new(self, |s: &$typ, _: &_, _: &_| {s.to_string()}).into()
                }
            }
            impl Into<StringState> for $typ {
                fn into(self) -> StringState {
                    MapOwnedState::new(ValueState::new(self), |s: &$typ, _: &_, _: &_| {s.to_string()}).into()
                }
            }
            impl Into<StringState> for Box<ValueState<$typ>> {
                fn into(self) -> StringState {
                    MapOwnedState::new(WidgetState::new(self), |s: &$typ, _: &_, _: &_| {s.to_string()}).into()
                }
            }
            impl Into<StringState> for Box<LocalState<$typ>> {
                fn into(self) -> StringState {
                    MapOwnedState::new(WidgetState::new(self), |s: &$typ, _: &_, _: &_| {s.to_string()}).into()
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