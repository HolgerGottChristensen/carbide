use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;

use crate::prelude::Environment;
use crate::state::*;
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};

pub trait State<T, GS>: DynClone + Deref<Target=T> + DerefMut + Debug where T: StateContract, GS: GlobalStateContract {
    /// This should take the state from the environment to hold locally in the implementer.
    /// Other implementations could also take copies of global_state, and apply mappings to other
    /// states.
    /// This will always be the first thing called for the states of a widget when retrieving an
    /// event. This makes sure the local and other states are always up to date when recieving
    /// an event.
    fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);

    /// This releases local state from the widget back into the environment. This is called
    /// after the event has been processed in a widget, but before the children of the widget
    /// has is being processed. This makes sure the state is always available for the widget
    /// being processed.
    fn release_state(&mut self, env: &mut Environment<GS>);
}

dyn_clone::clone_trait_object!(<T: StateContract, GS: GlobalStateContract> State<T, GS>);

/*impl<T: StateContract, GS: GlobalStateContract> State<T, GS, Target=T> for Box<dyn State<T, GS, Target=T>> {
    fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &mut Rc<GS>) {
        self.deref_mut().capture_state(env, global_state);
    }

    fn release_state(&mut self, env: &mut Environment<GS>) {
        self.deref_mut().release_state(env);
    }
}*/

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