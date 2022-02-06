use std::fmt::Debug;
use carbide_core::prelude::{NewStateSync, Listenable, Listener};

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{MapRev, ReadState, State};
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

// Due to errors with lifetimes and closures it seems we are not able to use fn closures,
// because we can not provide any closures with valid lifetimes as mapping functions.
// See: https://github.com/rust-lang/rust/issues/86921
// Because we cant use closures, we store a generic value of stuff, that is parsed to the
// mapping function. This should make you able to use it as if it was a closure, by capturing
// values manually, but is for sure more inconvenient.

#[derive(Clone)]
pub struct MapState<FROM, TO, VALUE>
    where
        FROM: StateContract,
        TO: StateContract,
        VALUE: StateContract
{
    state: TState<FROM>,
    inner_value: VALUE,
    map: for<'r, 's> fn(&'r FROM, VALUE) -> &'r TO,
    map_mut: for<'r, 's> fn(&'r mut FROM, VALUE) -> &'r mut TO,
    map_rev: Option<Box<dyn MapRev<FROM, TO>>>,
}

impl<FROM: StateContract, TO: StateContract, VALUE: StateContract> MapState<FROM, TO, VALUE> {
    pub fn new<S, M1: MapRev<FROM, TO>>(
        state: S,
        value: VALUE,
        map: for<'r, 's> fn(&'r FROM, VALUE) -> &'r TO,
        map_mut: for<'r, 's> fn(&'r mut FROM, VALUE) -> &'r mut TO,
        map_rev: M1,
    ) -> Self
        where
            S: Into<TState<FROM>>,
    {
        MapState {
            state: state.into(),
            inner_value: value,
            map,
            map_mut,
            map_rev: Some(Box::new(map_rev)),
        }
    }
}

impl<FROM: StateContract, TO: StateContract, VALUE: StateContract> NewStateSync for MapState<FROM, TO, VALUE> {
    fn sync(&mut self, env: &mut Environment) {
        self.state.sync(env)
    }
}

impl<FROM: StateContract, TO: StateContract, VALUE: StateContract> Listenable<TO> for MapState<FROM, TO, VALUE> {
    fn subscribe(&self, subscriber: Box<dyn Listener<TO>>) {
        todo!()
    }
}

impl<FROM: StateContract, TO: StateContract, VALUE: StateContract> ReadState<TO> for MapState<FROM, TO, VALUE> {
    fn value(&self) -> ValueRef<TO> {
        ValueRef::map(self.state.value(), |a| { (self.map)(a, self.inner_value.clone()) })
    }
}


impl<FROM: StateContract, TO: StateContract, VALUE: StateContract> State<TO> for MapState<FROM, TO, VALUE> {
    fn value_mut(&mut self) -> ValueRefMut<TO> {
        let val = self.inner_value.clone();
        let function = self.map_mut;
        ValueRefMut::map(self.state.value_mut(), |a| { function(a, val) })
    }

    fn set_value(&mut self, value: TO) {
        if let Some(map_rev) = &self.map_rev {
            let from: Option<FROM> = map_rev(&value);
            if let Some(from) = from {
                self.state.set_value(from);
            }
        }

        let val = self.inner_value.clone();
        let function = self.map_mut;
        *ValueRefMut::map(self.state.value_mut(), |a| { function(a, val) }) = value;
    }

    fn notify(&self) {
        todo!()
    }
}

impl<FROM: StateContract, TO: StateContract, VALUE: StateContract> Debug for MapState<FROM, TO, VALUE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract, TO: StateContract, VALUE: StateContract> Into<TState<TO>>
for MapState<FROM, TO, VALUE>
{
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}