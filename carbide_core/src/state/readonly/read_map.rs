use crate::state::StateContract;

pub trait ReadMap<FROM, TO>: Clone + 'static where FROM: StateContract, TO: StateContract {
    fn map(&self, from: &FROM) -> TO;
}

impl<FROM: StateContract, TO: StateContract, K> ReadMap<FROM, TO> for K where K: Fn(&FROM) -> TO + Clone + 'static {
    fn map(&self, from: &FROM) -> TO {
        self(from)
    }
}