use std::fmt::Debug;
use carbide_core::prelude::NewStateSync;

use crate::environment::Environment;
use crate::prelude::StateContract;
use crate::state::{InnerState, ReadState, RState, ValueCell};
use crate::state::readonly::{ReadMap, ReadWidgetState};
use crate::state::util::value_cell::ValueRef;

#[derive(Clone)]
pub struct ReadMapState<FROM, TO, MAP>
    where
        FROM: StateContract,
        TO: StateContract,
        MAP: ReadMap<FROM, TO>,
{
    state: RState<FROM>,
    value: InnerState<TO>,
    map: MAP
}

impl<FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO>> ReadMapState<FROM, TO, MAP> {
    pub fn new<M1: Into<RState<FROM>>>(state: M1, map: MAP) -> RState<TO> {
        let state = state.into();
        let value = map.map(&*state.value());

        let inner_state = InnerState::new(ValueCell::new(value));

        let res = ReadMapState {
            state: state.clone(),
            value: inner_state.clone(),
            map
        };

        res.into()
    }
}

impl<FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO>> NewStateSync for ReadMapState<FROM, TO, MAP> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let updated = self.state.sync(env);

        if updated {
            *self.value.borrow_mut() = self.map.map(&*self.state.value());
        }

        updated
    }
}


impl<FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO>> ReadState<TO> for ReadMapState<FROM, TO, MAP> {
    fn value(&self) -> ValueRef<TO> {
        self.value.borrow()
    }
}

impl<FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO>> Debug for ReadMapState<FROM, TO, MAP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO>> Into<RState<TO>> for ReadMapState<FROM, TO, MAP> {
    fn into(self) -> RState<TO> {
        ReadWidgetState::new(Box::new(self))
    }
}
