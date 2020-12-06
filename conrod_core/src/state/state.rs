use std::ops::{Deref, DerefMut};
use std::borrow::BorrowMut;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;
use std::convert::TryInto;
use widget::common_widget::CommonWidget;
use uuid::Uuid;

#[derive(Debug)]
pub struct State<T> where T: Clone + Debug {
    pub id: String,
    pub value: T,
}

impl<T: Clone + Debug> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            value: self.value.clone()
        }
    }
}

impl<T: Clone + Debug> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Clone + Debug> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Clone + Debug> State<T> {
    pub fn new(name: &str, val: &T) -> Self {
        State {
            id: name.to_string(),
            value: val.clone()
        }
    }
}



pub type StateList<T: Clone> = Vec<(String, T)>;

pub trait GetState<T> {
    fn get_state(&self, key: &String) -> Option<&(String, T)>;
    fn replace_state(&mut self, val: (String, T));
}

impl<T> GetState<T> for StateList<T> {
    fn get_state(&self, key: &String) -> Option<&(String, T)> {
        self.iter().find(|(try_key, state)| key.eq(try_key))
    }

    fn replace_state(&mut self, val: (String, T)) {
        let (id, val) = val;
        self.retain(|(i, s)| {
            id.ne(i)
        });
        self.push((id, val));
    }
}

#[derive(Clone, Debug)]
pub enum DefaultState {
    String(String),
    UuidList(Vec<Uuid>),
    Uuid(Uuid),
    U32(u32)
}

impl Into<(String, DefaultState)> for State<Uuid> {
    fn into(self) -> (String, DefaultState) {
        (self.id, DefaultState::Uuid(self.value))
    }
}

impl Into<(String, DefaultState)> for State<Vec<Uuid>> {
    fn into(self) -> (String, DefaultState) {
        (self.id, DefaultState::UuidList(self.value))
    }
}

impl Into<(String, DefaultState)> for State<String> {
    fn into(self) -> (String, DefaultState) {
        (self.id, DefaultState::String(self.value))
    }
}

impl Into<(String, DefaultState)> for State<u32> {
    fn into(self) -> (String, DefaultState) {
        (self.id, DefaultState::U32(self.value))
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

impl Into<State<String>> for (String, DefaultState) {
    fn into(self) -> State<String> {
        let (id, state) = self;
        match state {
            DefaultState::String(n) => {
                State::new(&id, &n)
            }
            _ => panic!()
        }
    }
}

impl Into<State<u32>> for (String, DefaultState) {
    fn into(self) -> State<u32> {
        let (id, state) = self;
        match state {
            DefaultState::U32(n) => {
                State::new(&id, &n)
            }
            _ => panic!()
        }
    }
}

impl Into<State<Vec<Uuid>>> for (String, DefaultState) {
    fn into(self) -> State<Vec<Uuid>> {
        let (id, state) = self;
        match state {
            DefaultState::UuidList(n) => {
                State::new(&id, &n)
            }
            _ => panic!()
        }
    }
}

impl Into<State<Uuid>> for (String, DefaultState) {
    fn into(self) -> State<Uuid> {
        let (id, state) = self;
        match state {
            DefaultState::Uuid(n) => {
                State::new(&id, &n)
            }
            _ => panic!()
        }
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