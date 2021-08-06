use crate::prelude::Environment;
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::widget::common_widget::CommonWidget;

pub trait StateSync<GS>: CommonWidget<GS> where GS: GlobalStateContract {
    fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);
    fn release_state(&mut self, env: &mut Environment<GS>);
}

pub trait NoLocalStateSync {}

impl<GS: GlobalStateContract, T> StateSync<GS> for T where T: NoLocalStateSync + CommonWidget<GS> {
    fn capture_state(&mut self, _: &mut Environment<GS>, _: &GlobalStateContainer<GS>) {}

    fn release_state(&mut self, _: &mut Environment<GS>) {}
}