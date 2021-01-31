use std::fmt::Debug;

use bitflags::_core::fmt::Formatter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{from_ron, to_ron};
use crate::state::global_state::GlobalState;
use dyn_clone::DynClone;
use std::ops::{DerefMut, Deref};


pub trait State<T, GS>: DynClone where T: Serialize + Clone + Debug, GS: GlobalState {
    fn get_value_mut(&mut self, global_state: &mut GS) -> &mut T;
    fn get_value(&mut self, global_state: &GS) -> &T;
    fn get_latest_value(&self) -> &T;
    fn get_latest_value_mut(&mut self) -> &mut T;
    fn get_key(&self) -> Option<&String>;
}

dyn_clone::clone_trait_object!(<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS>);


impl<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS> for Box<dyn State<T, GS>> {
    fn get_value_mut(&mut self, global_state: &mut GS) -> &mut T {
        self.deref_mut().get_value_mut(global_state)
    }

    fn get_value(&mut self, global_state: &GS) -> &T {
        self.deref_mut().get_value(global_state)
    }

    fn get_latest_value(&self) -> &T {
        self.deref().get_latest_value()
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        self.deref_mut().get_latest_value_mut()
    }

    fn get_key(&self) -> Option<&String> {
        self.deref().get_key()
    }
}

impl<T: Serialize + Clone + Debug, GS: GlobalState> State<T, GS> for CommonState<T, GS> {
    fn get_value_mut(&mut self, global_state: &mut GS) -> &mut T {
        self.get_value_mut(global_state)
    }

    fn get_value(&mut self, global_state: &GS) -> &T {
        self.get_value(global_state)
    }

    fn get_latest_value(&self) -> &T {
        self.get_latest_value()
    }

    fn get_latest_value_mut(&mut self) -> &mut T {
        self.get_latest_value_mut()
    }

    fn get_key(&self) -> Option<&String> {
        self.get_key()
    }
}



#[derive(Clone)]
pub enum CommonState<T, GS> where T: Serialize + Clone + Debug, GS: GlobalState {
    LocalState { id: String, value: T },
    Value { value: T },
    GlobalState {
        function: fn(state: &GS) -> T,
        function_mut: Option<fn(state: &mut GS) -> T>,
        latest_value: T
    }
    // KeyedEnvironmentState
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
        }


    }
}

impl<T: Serialize + Clone + Debug, U: GlobalState> CommonState<T, U> {
    pub fn get_value_mut(&mut self, global_state: &mut U) -> &mut T {
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
        }
    }

    pub fn get_value(&mut self, global_state: &U) -> &T {
        match self {
            CommonState::LocalState { value, .. } => {value}
            CommonState::Value { value } => {value}
            CommonState::GlobalState { latest_value, function, .. } => {
                *latest_value = function(global_state).clone();
                latest_value
            }
        }
    }

    pub fn get_latest_value(&self) -> &T {
        match self {
            CommonState::LocalState { value, .. } => {value}
            CommonState::Value { value } => {value}
            CommonState::GlobalState { latest_value, .. } => {
                latest_value
            }
        }
    }

    pub fn get_latest_value_mut(&mut self) -> &mut T {
        match self {
            CommonState::LocalState { value, .. } => {value}
            CommonState::Value { value } => {value}
            CommonState::GlobalState { latest_value, .. } => {
                latest_value
            }
        }
    }

    pub fn get_key(&self) -> Option<&String> {
        match self {
            CommonState::LocalState {id, ..} => Some(id),
            _ => None
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

impl<T: Clone + Debug + Serialize, S: GlobalState> CommonState<T, S> {
    pub fn new(val: &T) -> Self {
        CommonState::Value {
            value: val.clone()
        }
    }

    pub fn new_local(key: &str, val: &T) -> Self {
        CommonState::LocalState {
            id: key.to_string(),
            value: val.clone(),
        }
    }

    pub fn new_local_with_key(val: &T) -> Self {
        CommonState::LocalState {
            id: Uuid::new_v4().to_string(),
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
                let key = &*id;
                match self.iter().find(|(try_key, _state)| key.eq(try_key)) {
                    None => (),
                    Some((_, val)) => {
                        *value = from_ron(&val).unwrap();
                    }
                }
            }
            CommonState::Value { .. } => {}
            CommonState::GlobalState {function, latest_value, ..} => {
                *latest_value = function(global_state).clone()
            }
        }

    }

    fn replace_state<T: Serialize + Clone + Debug, U: GlobalState>(&mut self, val: CommonState<T,U>) {
        match val {
            CommonState::LocalState { id, value } => {
                let val = to_ron(&value).unwrap();
                self.retain(|(i, _s)| {
                    id.ne(i)
                });
                self.push((id, val));
            }
            CommonState::Value { .. } => {}
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