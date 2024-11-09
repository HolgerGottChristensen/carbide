use crate::environment::{Environment, EnvironmentStack};

pub trait StateSync {
    /// This will get called automatically if the field storing this is marked #\[state\] and is
    /// inside a widget. This should get called before the widget receives events but after its
    /// parent widget receives its event. If the sync resulted in a state updating, the method
    /// will return true.
    #[allow(unused_variables)]
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        false
    }
}
