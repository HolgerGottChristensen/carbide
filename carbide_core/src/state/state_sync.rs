use crate::prelude::Environment;
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::widget::common_widget::CommonWidget;

pub trait StateSync: CommonWidget {
    fn capture_state(&mut self, env: &mut Environment);
    fn release_state(&mut self, env: &mut Environment);
}

pub trait NoLocalStateSync {}

impl<T> StateSync for T where T: NoLocalStateSync + CommonWidget {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}
}