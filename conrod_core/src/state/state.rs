use std::ops::{Deref, DerefMut};
use std::borrow::BorrowMut;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;
use std::convert::TryInto;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use ::{from_ron, to_ron};

#[derive(Debug)]
pub struct State<T> where T: Serialize + Clone + Debug {
    pub id: String,
    pub value: T,
}

impl<T: Clone + Debug + Serialize> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            value: self.value.clone()
        }
    }
}

impl<T: Clone + Debug + Serialize> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Clone + Debug + Serialize> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Clone + Debug + Serialize> State<T> {
    pub fn new(name: &str, val: &T) -> Self {
        State {
            id: name.to_string(),
            value: val.clone()
        }
    }
}

pub type StateList = Vec<(String, String)>;

pub trait GetState {
    fn update_local_state<'a, T: Deserialize<'a> + Serialize + Clone + Debug>(&'a self, state: &mut State<T>);
    fn replace_state<T: Serialize + Clone + Debug>(&mut self, val: State<T>);
}

impl GetState for StateList {
    fn update_local_state<'a, T: Deserialize<'a> + Serialize + Clone + Debug>(&'a self, state: &mut State<T>) {
        let key = &state.id;
        match self.iter().find(|(try_key, state)| key.eq(try_key)) {
            None => (),
            Some((_, value)) => {
                state.value = from_ron(&value).unwrap();
            }
        }
    }

    fn replace_state<T: Serialize + Clone + Debug>(&mut self, val: State<T>) {
        let id = val.id;
        let val = to_ron(&val.value).unwrap();
        self.retain(|(i, s)| {
            id.ne(i)
        });
        self.push((id, val));
    }
}

impl Into<State<Uuid>> for Uuid {
    fn into(self) -> State<Uuid> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl Into<State<Vec<Uuid>>> for Vec<Uuid> {
    fn into(self) -> State<Vec<Uuid>> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl Into<State<u32>> for u32 {
    fn into(self) -> State<u32> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl Into<State<String>> for String {
    fn into(self) -> State<String> {
        State::new(&Uuid::new_v4().to_string(), &self)
    }
}

impl Into<State<String>> for &str {
    fn into(self) -> State<String> {
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