use std::fmt::Debug;

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{MapRev, State};
use crate::state::value_cell::{ValueRef, ValueRefMut};
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
        FROM: StateContract + 'static,
        TO: StateContract + 'static,
        VALUE: StateContract + 'static
{
    state: TState<FROM>,
    inner_value: VALUE,
    map: for<'r, 's> fn(&'r FROM, VALUE) -> &'r TO,
    map_mut: for<'r, 's> fn(&'r mut FROM, VALUE) -> &'r mut TO,
    map_rev: Option<Box<dyn MapRev<FROM, TO>>>,
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static, VALUE: StateContract + 'static> MapState<FROM, TO, VALUE> {
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

impl<FROM: StateContract + 'static, TO: StateContract + 'static, VALUE: StateContract + 'static> State<TO> for MapState<FROM, TO, VALUE> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.state.capture_state(env)
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.state.release_state(env)
    }

    fn value(&self) -> ValueRef<TO> {
        ValueRef::map(self.state.value(), |a| { (self.map)(a, self.inner_value.clone()) })
    }

    fn value_mut(&mut self) -> ValueRefMut<TO> {
        let val = self.inner_value.clone();
        let function = self.map_mut;
        ValueRefMut::map(self.state.value_mut(), |a| { function(a, val) })
    }

    fn set_value(&mut self, value: TO) {
        if let Some(map_rev) = &self.map_rev {
            let from: FROM = map_rev(&value);
            self.state.set_value(from);
        }

        let val = self.inner_value.clone();
        let function = self.map_mut;
        *ValueRefMut::map(self.state.value_mut(), |a| { function(a, val) }) = value;
    }
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static, VALUE: StateContract + 'static> Debug for MapState<FROM, TO, VALUE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract + 'static, TO: StateContract + 'static, VALUE: StateContract + 'static> Into<TState<TO>>
for MapState<FROM, TO, VALUE>
{
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}