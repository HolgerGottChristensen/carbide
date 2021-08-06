use std::cell::{Ref, RefCell};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::prelude::Environment;
use crate::state::{State, StateContract, TState};
use crate::state::widget_state::WidgetState;

// The global state needs to implement clone because the widgets do, and for them to be clone
// All the generic types need to implement it as well. The global state should never in practise
// be cloned, because that would most likely be very expensive.
pub trait GlobalStateContract: 'static + Clone + std::fmt::Debug {}

impl<T> GlobalStateContract for T where T: 'static + Clone + std::fmt::Debug {}

pub type GlobalStateContainer<GS: GlobalStateContract> = Rc<RefCell<GS>>;

#[derive(Clone)]
pub struct GState<T, GS> where T: StateContract + 'static, GS: GlobalStateContract {
    function: fn(state: &GS) -> &T,
    function_mut: fn(state: &mut GS) -> &mut T,
    global_state: Option<GlobalStateContainer<GS>>,
}

struct FooGuard<T: 'static> {
    guard: Ref<'static, T>,
}

impl<T: StateContract, GS: GlobalStateContract> Deref for GState<T, GS> {
    type Target = FooGuard<T>;

    fn deref(&self) -> &FooGuard<T> {
        let state = self.global_state.as_ref().expect("No global state in state.");
        let inner = Ref::map(state.borrow(), |gs| (self.function)(gs));
        &inner
    }
}

impl<T: StateContract, GS: GlobalStateContract> DerefMut for GState<T, GS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        (self.function_mut)(&mut *self.global_state.expect("No global state in state.").borrow_mut())
    }
}

impl<T: StateContract, GS: GlobalStateContract> GState<T, GS> {
    pub fn new(
        function: fn(state: &GS) -> &T,
        function_mut: fn(state: &mut GS) -> &mut T,
    ) -> Box<Self> {
        Box::new(GState {
            function,
            function_mut,
            global_state: None,
        })
    }
}

impl<T: StateContract, GS: GlobalStateContract> State<T, GS> for GState<T, GS> {
    fn capture_state(&mut self, _: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        if let Some(_) = &self.global_state {} else {
            self.global_state = Some(global_state.clone());
        }
    }

    fn release_state(&mut self, _: &mut Environment<GS>) {}
}

impl<T: StateContract, GS: GlobalStateContract> Debug for GState<T, GS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::GlobalState")
            .field("value", self.deref())
            .finish()
    }
}

impl<T: StateContract + 'static, GS: GlobalStateContract> Into<TState<T, GS>> for Box<GState<T, GS>> {
    fn into(self) -> TState<T, GS> {
        WidgetState::new(self)
    }
}