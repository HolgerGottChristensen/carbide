use crate::environment::{EnvironmentStack, Key};
use crate::focus::Refocus;

#[derive(Debug)]
pub struct FocusManager {
    focus_request: Option<Refocus>
}

impl FocusManager {
    pub fn new() -> FocusManager {
        FocusManager {
            focus_request: None,
        }
    }

    pub fn get(env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut FocusManager)) {
        if let Some(manager) = env_stack.get_mut::<FocusManager>() {
            f(manager)
        }
    }

    pub fn request_focus(&mut self, refocus: Refocus) {
        self.focus_request = Some(refocus)
    }

    pub fn requested_focus(&mut self) -> Option<Refocus> {
        self.focus_request.take()
    }
}

impl Key for FocusManager {
    type Value = FocusManager;
}