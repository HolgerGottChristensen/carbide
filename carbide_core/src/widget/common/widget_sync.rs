use crate::environment::{EnvironmentStack};

pub trait WidgetSync {
    #[allow(unused_variables)]
    fn sync(&mut self, env: &mut EnvironmentStack) {}
}
