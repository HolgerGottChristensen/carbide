use crate::environment::Environment;

pub trait WidgetSync {
    #[allow(unused_variables)]
    fn sync(&mut self, env: &mut Environment) {}
}
