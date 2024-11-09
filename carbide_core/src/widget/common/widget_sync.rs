use crate::environment::{Environment, EnvironmentStack};

pub trait WidgetSync {
    #[allow(unused_variables)]
    fn sync(&mut self, env: &mut EnvironmentStack) {}
}
