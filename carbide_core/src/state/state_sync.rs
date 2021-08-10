use crate::prelude::Environment;
use crate::widget::CommonWidget;

pub trait StateSync: CommonWidget {
    fn capture_state(&mut self, env: &mut Environment);
    fn release_state(&mut self, env: &mut Environment);
}
