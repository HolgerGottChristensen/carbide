use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use bitflags::_core::fmt::Formatter;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{from_ron, to_ron};
use crate::state::*;
use crate::state::environment::Environment;
use crate::state::global_state::GlobalState;
use crate::state::mapped_state::MappedState;
use crate::state::state_key::StateKey;
use crate::widget::widget_state::WidgetState;

pub trait State<T, GS>: DynClone where T: Serialize + Clone + Debug, GS: GlobalState {
    fn get_value_mut<'a>(&'a mut self, env: &'a mut Environment<GS>, global_state: &'a mut GS) -> &'a mut T;
    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &T;
    fn get_latest_value(&self) -> &T;
    fn get_latest_value_mut(&mut self) -> &mut T;
    fn get_key(&self) -> Option<&StateKey>;
    fn update_dependent_states(&mut self, env: &Environment<GS>);
    fn insert_dependent_states(&self, env: &mut Environment<GS>);
}


impl<T: Serialize + Clone + Debug, GS: GlobalState> Debug for dyn State<T, GS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The latest value printed is not necessarily the same as the current value.
        write!(f, "State with latest value: {:?}", *self.get_latest_value())
    }
}

dyn_clone::clone_trait_object!(<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS>);


impl<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS> for Box<dyn State<T, GS>> {
    fn get_value_mut<'a>(&'a mut self, env: &'a mut Environment<GS>, global_state: &'a mut GS) -> &mut T {
        self.deref_mut().get_value_mut(env, global_state)
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &T {
        self.deref_mut().get_value(env, global_state)
    }

    fn get_latest_value(&self) -> &T {
        self.deref().get_latest_value()
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        self.deref_mut().get_latest_value_mut()
    }

    fn get_key(&self) -> Option<&StateKey> {
        self.deref().get_key()
    }

    fn update_dependent_states(&mut self, env: &Environment<GS>) {
        self.deref_mut().update_dependent_states(env)
    }

    fn insert_dependent_states(&self, env: &mut Environment<GS>) {
        self.deref().insert_dependent_states(env)
    }
}

// TODO: Split into different structs.
#[derive(Clone)]
pub enum CommonState<T, GS> where T: Serialize + Clone + Debug, GS: GlobalState {
    LocalState { id: StateKey, value: T },
    Value { value: T },
    GlobalState {
        function: fn(state: &GS) -> &T,
        function_mut: fn(state: &mut GS) -> &mut T,
        latest_value: T,
    },
    EnvironmentState {
        function: fn(env: &Environment<GS>) -> T,
        function_mut: Option<fn(env: &mut Environment<GS>) -> &mut T>,
        latest_value: T,
    },
}

impl<T: Serialize + Clone + Debug, GS: GlobalState> CommonState<T, GS> {

    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
    }
}

impl<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS> for CommonState<T, GS> {
    fn get_value_mut<'a>(&'a mut self, env: &'a mut Environment<GS>, global_state: &'a mut GS) -> &'a mut T {
        match self {
            CommonState::LocalState { value, .. } => { value }
            CommonState::Value { value } => { value }
            CommonState::GlobalState { latest_value, function_mut, .. } => {
                *latest_value = function_mut(global_state).clone();

                function_mut(global_state)
            }
            CommonState::EnvironmentState { latest_value, function_mut, function } => {
                if let Some(function_mut) = function_mut {
                    *latest_value = function_mut(env).clone();
                    function_mut(env)
                } else {
                    *latest_value = function(env);
                    latest_value
                }
            }
        }
    }

    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &T {
        match self {
            CommonState::LocalState { value, .. } => {value}
            CommonState::Value { value } => {value}
            CommonState::GlobalState { latest_value, function, .. } => {
                *latest_value = function(global_state).clone();
                latest_value
            }
            CommonState::EnvironmentState { latest_value, function, .. } => {
                *latest_value = function(env).clone();
                latest_value
            }
        }
    }

    fn get_latest_value(&self) -> &T {
        match self {
            CommonState::LocalState { value, .. } => {value}
            CommonState::Value { value } => {value}
            CommonState::GlobalState { latest_value, .. } => {
                latest_value
            }
            CommonState::EnvironmentState { latest_value, .. } => {
                latest_value
            }
        }
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        match self {
            CommonState::LocalState { value, .. } => {value}
            CommonState::Value { value } => {value}
            CommonState::GlobalState { latest_value, .. } => {
                latest_value
            }
            CommonState::EnvironmentState { latest_value, .. } => {
                latest_value
            }
        }
    }

    fn get_key(&self) -> Option<&StateKey> {
        match self {
            CommonState::LocalState {id, ..} => Some(id),
            _ => None
        }
    }

    fn update_dependent_states(&mut self, _: &Environment<GS>) {}

    fn insert_dependent_states(&self, _: &mut Environment<GS>) {}
}





impl<T: Serialize + Clone + Debug, U: GlobalState> Debug for CommonState<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommonState::LocalState { id, value } => {
                f.debug_struct("State::LocalState")
                    .field("id", id)
                    .field("value", value)
                    .finish()
                //Todo: .finish_non_exhaustive() Change to this when updating to rust 1.53
            }
            CommonState::Value { value } => {
                f.debug_struct("State::Value")
                    .field("value", value)
                    .finish()
            }
            CommonState::GlobalState { latest_value, .. } => {
                f.debug_struct("State::GlobalState")
                    .field("latest_value", latest_value)
                    .finish()
            }
            CommonState::EnvironmentState { latest_value, .. } => {
                f.debug_struct("State::EnvironmentState")
                    .field("latest_value", latest_value)
                    .finish()
            }
        }


    }
}


impl<T: Clone + Debug + Serialize + 'static, S: GlobalState> CommonState<T, S> {
    pub fn new(val: &T) -> Self {
        CommonState::Value {
            value: val.clone()
        }
    }

    pub fn new_local_with_key(val: &T) -> Self {
        CommonState::LocalState {
            id: StateKey::String(Uuid::new_v4().to_string()),
            value: val.clone(),
        }
    }
}

/*pub trait EnvFn<T: Serialize + Clone + Debug, GS: GlobalState>: FnOnce(&Environment<GS>) -> T + DynClone {}
pub trait EnvFnMut<T: Serialize + Clone + Debug, GS: GlobalState>: FnOnce(&mut Environment<GS>) -> T + DynClone {}

impl<T: Serialize + Clone + Debug, U: FnOnce(&Environment<GS>) -> T + DynClone, GS: GlobalState> EnvFn<T, GS> for U {}
impl<T: Serialize + Clone + Debug, U: FnOnce(&mut Environment<GS>) -> T + DynClone, GS: GlobalState> EnvFnMut<T, GS> for U {}*/

/*pub type LocalStateList = Vec<(String, String)>;

pub trait GetState {
    fn update_local_state<'a, T: Deserialize<'a> + Serialize + Clone + Debug, U: GlobalState>(&'a self, state: &mut CommonState<T, U>, global_state: &U);
    fn replace_state<T: Serialize + Clone + Debug, U: GlobalState>(&mut self, val: CommonState<T, U>);
}

impl GetState for LocalStateList {
    fn update_local_state<'a, T: Deserialize<'a> + Serialize + Clone + Debug, U: GlobalState>(&'a self, state: &mut CommonState<T, U>, global_state: &U) {
        match state {
            CommonState::LocalState { id, value } => {
                if let StateKey::String(key) = &*id {
                    match self.iter().find(|(try_key, _state)| key.eq(try_key)) {
                        None => (),
                        Some((_, val)) => {
                            *value = from_ron(&val).unwrap();
                        }
                    }
                }
            }
            CommonState::Value { .. } => {}
            CommonState::GlobalState {function, latest_value, ..} => {
                *latest_value = function(global_state).clone()
            }
            CommonState::EnvironmentState {..} => {}
        }

    }

    fn replace_state<T: Serialize + Clone + Debug, U: GlobalState>(&mut self, val: CommonState<T,U>) {
        match val {
            CommonState::LocalState { id, value } => {
                if let StateKey::String(key) = id {
                    let val = to_ron(&value).unwrap();
                    self.retain(|(i, _s)| {
                        key.ne(i)
                    });
                    self.push((key, val));
                }

            }
            _ => {}
        }

    }
}*/

impl<GS: GlobalState> Into<CommonState<Uuid, GS>> for Uuid {
    fn into(self) -> CommonState<Uuid, GS> {
        CommonState::new(&self)
    }
}

impl<GS: GlobalState> Into<CommonState<Vec<Uuid>, GS>> for Vec<Uuid> {
    fn into(self) -> CommonState<Vec<Uuid>, GS> {
        CommonState::new(&self)
    }
}

impl<GS: GlobalState> Into<CommonState<u32, GS>> for u32 {
    fn into(self) -> CommonState<u32, GS> {
        CommonState::new(&self)
    }
}

impl<GS: GlobalState> Into<CommonState<f64, GS>> for f64 {
    fn into(self) -> CommonState<f64, GS> {
        CommonState::new(&self)
    }
}

impl<GS: GlobalState> Into<CommonState<String, GS>> for String {
    fn into(self) -> CommonState<String, GS> {
        CommonState::new(&self)
    }
}

impl<GS: GlobalState> Into<CommonState<String, GS>> for &str {
    fn into(self) -> CommonState<String, GS> {
        CommonState::new(&self.to_string())
    }
}

impl<GS: GlobalState> Into<CommonState<bool, GS>> for bool {
    fn into(self) -> CommonState<bool, GS> {
        CommonState::new(&self)
    }
}




impl<GS: GlobalState> Into<TState<Uuid, GS>> for Uuid {
    fn into(self) -> TState<Uuid, GS> {
        WidgetState::new(Box::new(CommonState::new(&self)))
    }
}

impl<GS: GlobalState> Into<TState<Vec<Uuid>, GS>> for Vec<Uuid> {
    fn into(self) -> TState<Vec<Uuid>, GS> {
        WidgetState::new(Box::new(CommonState::new(&self)))
    }
}

impl<GS: GlobalState> Into<U32State<GS>> for u32 {
    fn into(self) -> U32State<GS> {
        WidgetState::new(Box::new(CommonState::new(&self)))
    }
}

impl<GS: GlobalState> Into<UsizeState<GS>> for usize {
    fn into(self) -> UsizeState<GS> {
        WidgetState::new(Box::new(CommonState::new(&self)))
    }
}

impl<GS: GlobalState> Into<F64State<GS>> for f64 {
    fn into(self) -> F64State<GS> {
        WidgetState::new(Box::new(CommonState::new(&self)))
    }
}

impl<GS: GlobalState> Into<StringState<GS>> for String {
    fn into(self) -> StringState<GS> {
        WidgetState::new(Box::new(CommonState::new(&self)))
    }
}

impl<GS: GlobalState> Into<StringState<GS>> for &str {
    fn into(self) -> StringState<GS> {
        WidgetState::new(Box::new(CommonState::new(&self.to_string())))
    }
}

impl<GS: GlobalState> Into<BoolState<GS>> for bool {
    fn into(self) -> BoolState<GS> {
        WidgetState::new(Box::new(CommonState::new(&self)))
    }
}

impl<T: StateContract + 'static, GS: GlobalState> Into<TState<T, GS>> for CommonState<T, GS> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract + 'static, GS: GlobalState> Into<TState<T, GS>> for Box<CommonState<T, GS>> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(self)
    }
}


// Build a macro that expands: bind!(self.hejsa)
// To:  self.get_id() + ".hejsa"

// Mark fields as #[state]
// And automatically include these when sending state down to its children.
// Mark fields in the children as #[binding]


// Send state(vec) in each event call


// Below is for the thing calls state/binding

// Send state to the first child
// When an state-element is retrieved by a child remove the state-element from the state. (Maybe Omit if env obj)
// Return new state(vec) if modified.
// update parent state with this if it finds a modified state
// Send state(vec) to the next child (This will be the updated state, send from the other child.)

// After the event is done processing, run through the tree and make all the child state
// consistent with their parent states.

// Then we can layout (size and positioning)

// Then we can render.