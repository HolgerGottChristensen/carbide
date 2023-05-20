use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use carbide_core::state::state_sync::NewStateSync;
use carbide_derive::State;

use crate::environment::Environment;
use crate::state::{InnerState, ReadState, State, StateContract, TState};
use crate::state::util::value_cell::{ValueCell, ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

/// # Local state
/// The local state is used as a shared state between multiple widgets within the same widget tree.
/// When cloning this the inner state will be shared between the original and the clone.
/// The same is the case for the list of listeners.
///
/// Local state is [Listenable]. You are able to [Listenable::subscribe()] for notifications
/// whenever this state changes.
///
/// Local state does not need to do any updating when [NewStateSync::sync()] is called because
/// all state is stored directly within.
/// Also it does not depend on any other states and therefore the event can be ignored.
#[derive(Clone)]
//#[derive(Clone, State)]
pub struct LocalState<T>
where
    T: StateContract,
{
    /// The shared state
    inner_value: InnerState<T>,
}

impl<T: StateContract> LocalState<T> {
    /// Returns a new local state containing the value provided.
    /// Returns the local state wrapped within a WidgetState.
    pub fn new(value: T) -> TState<T> {
        WidgetState::Local(LocalState {
            inner_value: Rc::new(ValueCell::new(value)),
        })
    }

    /// Returns a new local state containing the value provided.
    /// Often you should use `new` when creating states, but this can be used to get the state
    /// within a box.
    fn new_raw(value: T) -> Box<Self> {
        Box::new(LocalState {
            inner_value: Rc::new(ValueCell::new(value)),
        })
    }
}

impl<T: StateContract> NewStateSync for LocalState<T> {
    fn sync(&mut self, _env: &mut Environment) -> bool {
        // TODO: find a smarter way to determine if local state has been updated.
        // I guess we can figuring it out by storing a frame number in the local state
        // and in the env, and then comparing and updating whenever this is called and set_value
        // is called.
        true
    }
}

impl<T: StateContract> ReadState for LocalState<T> {
    type T = T;
    fn value(&self) -> ValueRef<T> {
        self.inner_value.borrow()
    }
}

impl<T: StateContract> State for LocalState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.inner_value.borrow_mut()
    }

    fn set_value(&mut self, value: T) {
        *self.inner_value.borrow_mut() = value;
    }
}

impl<T: StateContract> Debug for LocalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalState")
            .field("value", &*self.value())
            .finish()
    }
}

/*impl<T, __Other_T, __Other_T_State> std::ops::Add<__Other_T_State> for LocalState<T>
    where
        T: StateContract,
        __Other_T: StateContract,
        __Other_T_State: State<__Other_T> + Clone + 'static,
        <T as std::ops::Add<__Other_T>>::Output: StateContract,
        T: StateContract + std::ops::Add<__Other_T>
{
    type Output = carbide_core::state::RMap2<T, __Other_T, <T as std::ops::Add<__Other_T>>::Output, LocalState<T>, __Other_T_State>;

    fn add(self, rhs: __Other_T_State) -> Self::Output {
        carbide_core::state::Map2::read_map(self, rhs, |val1: &T, val2: &__Other_T| { val1.clone() + val2.clone() })
    }
}*/
