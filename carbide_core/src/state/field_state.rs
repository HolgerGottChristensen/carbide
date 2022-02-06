use std::fmt::Debug;
use std::rc::Rc;
use carbide_core::prelude::{NewStateSync, Listenable};

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{InnerState, MapRev, ReadState, State, Listener, ValueCell};
use crate::state::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct FieldState<FROM, TO>
    where
        FROM: StateContract,
        TO: StateContract,
{
    state: TState<FROM>,
    map: for<'r, 's> fn(&'r FROM) -> &'r TO,
    map_mut: for<'r, 's> fn(&'r mut FROM) -> &'r mut TO,
    subscribers: InnerState<Vec<Box<dyn Listener<TO>>>>,
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
            subscribers: Rc::new(ValueCell::new(vec![]))
        };

        // Subscribe to the state to receive state changes from the parent state
        //state.subscribe(Box::new(res.clone()));

        res.into()
    }
}

impl<FROM: StateContract, TO: StateContract> NewStateSync for FieldState<FROM, TO> {
    fn sync(&mut self, env: &mut Environment) {
        self.state.sync(env)
    }
}

impl<FROM: StateContract, TO: StateContract> Listenable<TO> for FieldState<FROM, TO> {
    fn subscribe(&self, subscriber: Box<dyn Listener<TO>>) {
        self.subscribers.borrow_mut().push(subscriber)
    }
}

impl<FROM: StateContract, TO: StateContract> ReadState<TO> for FieldState<FROM, TO> {
    fn value(&self) -> ValueRef<TO> {
        let map = self.map;
        ValueRef::map(self.state.value(), |a| { map(a) })
    }

    /*fn value_changed(&mut self) {
        println!("Changed value to: {:?}", self);
        let mut subscribers = self.subscribers.borrow_mut();
        for subscriber in &mut *subscribers {
            subscriber.change()
        }
    }*/
}

impl<FROM: StateContract, TO: StateContract> State<TO> for FieldState<FROM, TO> {
    fn value_mut(&mut self) -> ValueRefMut<TO> {
        let map_mut = self.map_mut;
        ValueRefMut::map(self.state.value_mut(), |a| { map_mut(a) })
    }

    fn set_value(&mut self, value: TO) {
        let map_mut = self.map_mut;
        // TODO: When the dependent state is not LocalState or ValueState this is not good enough
        // The problem is that we would like to avoid using value_mut but at the same time not
        // loose all performance by cloning the whole dependent state when we want to set a field.
        *ValueRefMut::map(self.state.value_mut(), |a| { map_mut(a) }) = value;

        // Notify all the states dependent on the parent that the value has changed.
        //self.state.value_changed();
    }

    fn notify(&self) {
        todo!()
    }
}

impl<FROM: StateContract, TO: StateContract> Debug for FieldState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldState")
            .finish()
    }
}

impl<FROM: StateContract, TO: StateContract> Into<TState<TO>> for FieldState<FROM, TO> {
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}