use carbide_core::prelude::NewStateSync;
use std::fmt::Debug;

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;
use crate::state::{ReadState, State};

/// # FieldState
/// The FieldState is a state mapping that should be used to retrieve a field of a struct or enum
/// as a new state. This is done efficiently because we have two mapping functions that maps the
/// references to the original struct, to the references to the field.
///
/// You should almost never need to use this state directly. Instead you should use the `lens!`
/// macro as this will do all the dirty work of creating the mapping functions provided you
/// specify which field it should access.
///
/// This state is [Listenable] and can notify subscribers when the field changes. Notice that
/// currently the listeners are notified of any changes in the parent struct/enum. This means that
/// the field has not necessarily changed when the notification is sent. This behavior will most
/// likely change in the future, such that at least providing the possibility to only send
/// notifications when the field changes. However no notifications should be missed.
///
/// [Clone]s of this field state will share subscriber lists, but all other data is kept separate.
///
/// When [NewStateSync::sync()] events are received, it will delegate if further to the
/// parent state, making sure the parent is up to date. FieldState does not need to handle sync
/// itself.
#[derive(Clone)]
pub struct FieldState<FROM, TO>
where
    FROM: StateContract,
    TO: StateContract,
{
    /// The state containing the original data to get the field from
    state: TState<FROM>,
    /// The immutable reference mapping that can return a reference to the field when provided a
    /// reference to the struct
    map: for<'r, 's> fn(&'r FROM) -> &'r TO,
    /// The mutable reference mapping that can return a mutable reference to the field when
    /// provided with a mutable reference to the struct.
    map_mut: for<'r, 's> fn(&'r mut FROM) -> &'r mut TO,
}

impl<FROM: StateContract, TO: StateContract> FieldState<FROM, TO> {
    pub fn new<S: Into<TState<FROM>>>(
        state: S,
        map: for<'r, 's> fn(&'r FROM) -> &'r TO,
        map_mut: for<'r, 's> fn(&'r mut FROM) -> &'r mut TO,
    ) -> TState<TO> {
        let state = state.into();

        let res = FieldState {
            state: state.clone(),
            map,
            map_mut,
        };

        res.into()
    }
}

impl<FROM: StateContract, TO: StateContract> NewStateSync for FieldState<FROM, TO> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.state.sync(env)
    }
}

impl<FROM: StateContract, TO: StateContract> ReadState<TO> for FieldState<FROM, TO> {
    fn value(&self) -> ValueRef<TO> {
        let map = self.map;
        ValueRef::map(self.state.value(), |a| map(a))
    }
}

impl<FROM: StateContract, TO: StateContract> State<TO> for FieldState<FROM, TO> {
    fn value_mut(&mut self) -> ValueRefMut<TO> {
        let map_mut = self.map_mut;
        ValueRefMut::map(self.state.value_mut(), |a| map_mut(a))
    }

    fn set_value(&mut self, value: TO) {
        let map_mut = self.map_mut;
        *ValueRefMut::map(self.state.value_mut(), |a| map_mut(a)) = value;
    }
}

impl<FROM: StateContract, TO: StateContract> Debug for FieldState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract, TO: StateContract> Into<TState<TO>> for FieldState<FROM, TO> {
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}
