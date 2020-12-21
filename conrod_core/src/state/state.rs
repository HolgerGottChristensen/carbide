use std::ops::{Deref, DerefMut};
use std::borrow::BorrowMut;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;
use std::convert::TryInto;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use serde::{Serialize, Deserialize, Serializer};
use ::{from_ron, to_ron};
use bitflags::_core::fmt::Formatter;

#[derive(Clone)]
pub enum State<T, U> where T: Serialize + Clone + Debug, U: Clone {
    LocalState {id: String, value: T},
    Value {value: T},
    GlobalState {
        function: fn(state: &U) -> T,
        function_mut: Option<fn(state: &mut U) -> T>,
        latest_value: T
    },
    // KeyedEnvironmentState
}

impl<T: Serialize + Clone + Debug, U: Clone> Debug for State<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::LocalState { id, value } => {
                f.debug_struct("State::LocalState")
                    .field("id", id)
                    .field("value", value)
                    .finish()
            }
            State::Value { value } => {
                f.debug_struct("State::Value")
                    .field("value", value)
                    .finish()
            }
            State::GlobalState { latest_value, .. } => {
                f.debug_struct("State::GlobalState")
                    .field("latest_value", latest_value)
                    .finish()
            }
        }


    }
}

impl<T: Serialize + Clone + Debug, U: Clone> State<T, U> {
    pub fn get_value_mut(&mut self, global_state: &mut U) -> &mut T {
        match self {
            State::LocalState { value, .. } => {value}
            State::Value { value } => {value}
            State::GlobalState { latest_value, function_mut, function } => {
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
            State::LocalState { value, .. } => {value}
            State::Value { value } => {value}
            State::GlobalState { latest_value, function, .. } => {
                *latest_value = function(global_state).clone();
                latest_value
            }
        }
    }

    pub fn get_latest_value(&self) -> &T {
        match self {
            State::LocalState { value, .. } => {value}
            State::Value { value } => {value}
            State::GlobalState { latest_value, .. } => {
                latest_value
            }
        }
    }

    pub fn get_key(&self) -> Option<&String> {
        match self {
            State::LocalState {id, ..} => Some(id),
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

impl<T: Clone + Debug + Serialize, S: Clone> State<T, S> {
    pub fn new(name: &str, val: &T) -> Self {
        State::LocalState {
            id: name.to_string(),
            value: val.clone()
        }
    }
}

pub type LocalStateList = Vec<(String, String)>;

pub trait GetState {
    fn update_local_state<'a, T: Deserialize<'a> + Serialize + Clone + Debug, U: Clone>(&'a self, state: &mut State<T, U>, global_state: &U);
    fn replace_state<T: Serialize + Clone + Debug, U: Clone + Debug>(&mut self, val: State<T, U>);
}

impl GetState for LocalStateList {
    fn update_local_state<'a, T: Deserialize<'a> + Serialize + Clone + Debug, U: Clone>(&'a self, state: &mut State<T, U>, global_state: &U) {
        match state {
            State::LocalState { id, value } => {
                let key = &*id;
                match self.iter().find(|(try_key, state)| key.eq(try_key)) {
                    None => (),
                    Some((_, val)) => {
                        *value = from_ron(&val).unwrap();
                    }
                }
            }
            State::Value { .. } => {}
            State::GlobalState {function, latest_value, ..} => {
                *latest_value = function(global_state).clone()
            }
        }

    }

    fn replace_state<T: Serialize + Clone + Debug, U: Clone>(&mut self, val: State<T,U>) {
        match val {
            State::LocalState { id, value } => {
                let val = to_ron(&value).unwrap();
                self.retain(|(i, s)| {
                    id.ne(i)
                });
                self.push((id, val));
            }
            State::Value { .. } => {}
            _ => {}
        }

    }
}

impl<T: Clone> Into<State<Uuid, T>> for Uuid {
    fn into(self) -> State<Uuid, T> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl<T: Clone> Into<State<Vec<Uuid>, T>> for Vec<Uuid> {
    fn into(self) -> State<Vec<Uuid>, T> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl<T: Clone> Into<State<u32,T>> for u32 {
    fn into(self) -> State<u32,T> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl<T: Clone> Into<State<String, T>> for String {
    fn into(self) -> State<String, T> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl<T: Clone> Into<State<String, T>> for &str {
    fn into(self) -> State<String, T> {
        State::new(&Uuid::new_v4().to_string(), &self.to_string())
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