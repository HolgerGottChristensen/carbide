use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use bitflags::_core::fmt::Formatter;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::{from_ron, to_ron};
use crate::state::environment::Environment;
use crate::state::global_state::GlobalState;
use crate::state::mapped_state::MappedState;
use crate::state::state_key::StateKey;

pub trait State<T, GS>: DynClone where T: Serialize + Clone + Debug, GS: GlobalState {
    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut T;
    fn get_value(&mut self, env: &Environment<GS>, global_state: &GS) -> &T;
    fn get_latest_value(&self) -> &T;
    fn get_latest_value_mut(&mut self) -> &mut T;
    fn get_key(&self) -> Option<&StateKey>;
    fn update_dependent_states(&mut self, env: &Environment<GS>);
    fn insert_dependent_states(&self, env: &mut Environment<GS>);
}

pub trait StateExt<T: Serialize + Clone + Debug + DeserializeOwned + 'static, GS: GlobalState>: State<T, GS> + Sized + 'static {
    fn mapped<U: Serialize + Clone + Debug + 'static>(self, map: fn(&T) -> U) -> Box<dyn State<U, GS>> {
        let latest_value = self.get_latest_value().clone();
        MappedState::new(Box::new(self), map, map(&latest_value))
    }
}

impl<X: 'static, T: Serialize + Clone + Debug + DeserializeOwned + 'static, GS: GlobalState> StateExt<T, GS> for X where X: State<T, GS> {}

impl<T: Serialize + Clone + Debug, GS: GlobalState> Debug for dyn State<T, GS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The latest value printed is not necessarily the same as the current value.
        write!(f, "State with latest value: {:?}", *self.get_latest_value())
    }
}

dyn_clone::clone_trait_object!(<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS>);


impl<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS> for Box<dyn State<T, GS>> {
    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut T {
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

pub trait EnvFn<T: Serialize + Clone + Debug, GS: GlobalState>: FnOnce(&Environment<GS>) -> T + DynClone {}
pub trait EnvFnMut<T: Serialize + Clone + Debug, GS: GlobalState>: FnOnce(&mut Environment<GS>) -> T + DynClone {}

impl<T: Serialize + Clone + Debug, U: FnOnce(&Environment<GS>) -> T + DynClone, GS: GlobalState> EnvFn<T, GS> for U {}
impl<T: Serialize + Clone + Debug, U: FnOnce(&mut Environment<GS>) -> T + DynClone, GS: GlobalState> EnvFnMut<T, GS> for U {}

// TODO: Split into different structs.
#[derive(Clone)]
pub enum CommonState<T, GS> where T: Serialize + Clone + Debug, GS: GlobalState {
    LocalState { id: StateKey, value: T },
    Value { value: T },
    GlobalState {
        function: fn(state: &GS) -> T,
        function_mut: Option<fn(state: &mut GS) -> T>,
        latest_value: T
    },
    EnvironmentState {
        function: fn(env: &Environment<GS>) -> T,
        function_mut: Option<fn(env: &mut Environment<GS>) -> T>,
        latest_value: T
    },
}

impl<T: Serialize + Clone + Debug, GS: GlobalState> CommonState<T, GS> {

    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
    }
}

impl<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS> for CommonState<T, GS> {
    fn get_value_mut(&mut self, env: &mut Environment<GS>, global_state: &mut GS) -> &mut T {
        match self {
            CommonState::LocalState { value, .. } => {value}
            CommonState::Value { value } => {value}
            CommonState::GlobalState { latest_value, function_mut, function } => {
                if let Some(n) = function_mut {
                    *latest_value = n(global_state).clone();
                } else {
                    *latest_value = function(global_state).clone();
                }

                latest_value
            }
            CommonState::EnvironmentState { latest_value, function, function_mut } => {
                if let Some(n) = function_mut {
                    *latest_value = n(env).clone();
                } else {
                    *latest_value = function(env).clone();
                }

                latest_value
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

/*impl<T: Clone + Debug + Serialize> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            State::LocalState { value, .. } => {value}
            State::Value { value } => {value}
        }
    }
}

impl<T: Clone + Debug + Serialize> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            State::LocalState { value, .. } => {value}
            State::Value { value } => {value}
        }
    }
}*/

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

pub type LocalStateList = Vec<(String, String)>;

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
}

impl<T: GlobalState> Into<CommonState<Uuid, T>> for Uuid {
    fn into(self) -> CommonState<Uuid, T> {
        CommonState::new(&self)
    }
}

impl<T: GlobalState> Into<CommonState<Vec<Uuid>, T>> for Vec<Uuid> {
    fn into(self) -> CommonState<Vec<Uuid>, T> {
        CommonState::new(&self)
    }
}

impl<T: GlobalState> Into<CommonState<u32,T>> for u32 {
    fn into(self) -> CommonState<u32,T> {
        CommonState::new(&self)
    }
}

impl<T: GlobalState> Into<CommonState<f64,T>> for f64 {
    fn into(self) -> CommonState<f64,T> {
        CommonState::new(&self)
    }
}

impl<T: GlobalState> Into<CommonState<String, T>> for String {
    fn into(self) -> CommonState<String, T> {
        CommonState::new(&self)
    }
}

impl<T: GlobalState> Into<CommonState<String, T>> for &str {
    fn into(self) -> CommonState<String, T> {
        CommonState::new(&self.to_string())
    }
}

impl<T: GlobalState> Into<CommonState<bool, T>> for bool {
    fn into(self) -> CommonState<bool, T> {
        CommonState::new(&self)
    }
}




impl<T: GlobalState> Into<Box<dyn State<Uuid, T>>> for Uuid {
    fn into(self) -> Box<dyn State<Uuid, T>> {
        Box::new(CommonState::new(&self))
    }
}

impl<T: GlobalState> Into<Box<dyn State<Vec<Uuid>, T>>> for Vec<Uuid> {
    fn into(self) -> Box<dyn State<Vec<Uuid>, T>> {
        Box::new(CommonState::new(&self))
    }
}

impl<T: GlobalState> Into<Box<dyn State<u32, T>>> for u32 {
    fn into(self) -> Box<dyn State<u32, T>> {
        Box::new(CommonState::new(&self))
    }
}

impl<T: GlobalState> Into<Box<dyn State<usize, T>>> for usize {
    fn into(self) -> Box<dyn State<usize, T>> {
        Box::new(CommonState::new(&self))
    }
}

impl<T: GlobalState> Into<Box<dyn State<f64, T>>> for f64 {
    fn into(self) -> Box<dyn State<f64, T>> {
        Box::new(CommonState::new(&self))
    }
}

impl<T: GlobalState> Into<Box<dyn State<String, T>>> for String {
    fn into(self) -> Box<dyn State<String, T>> {
        Box::new(CommonState::new(&self))
    }
}

impl<T: GlobalState> Into<Box<dyn State<String, T>>> for &str {
    fn into(self) -> Box<dyn State<String, T>> {
        Box::new(CommonState::new(&self.to_string()))
    }
}

impl<T: GlobalState> Into<Box<dyn State<bool, T>>> for bool {
    fn into(self) -> Box<dyn State<bool, T>> {
        Box::new(CommonState::new(&self))
    }
}

impl<T: Serialize + Clone + Debug + 'static, GS: GlobalState> Into<Box<dyn State<T, GS>>> for CommonState<T, GS> {
    fn into(self) -> Box<dyn State<T, GS>> {
        Box::new(self)
    }
}

impl<T: Serialize + Clone + Debug + 'static, GS: GlobalState> Into<Box<dyn State<T, GS>>> for Box<CommonState<T, GS>> {
    fn into(self) -> Box<dyn State<T, GS>> {
        self
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