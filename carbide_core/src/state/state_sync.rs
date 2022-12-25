use dyn_clone::DynClone;

use crate::environment::Environment;
use crate::widget::CommonWidget;

pub trait StateSync: CommonWidget {
    fn capture_state(&mut self, env: &mut Environment);
    fn release_state(&mut self, env: &mut Environment);
}

pub trait NewStateSync: DynClone {
    /// This will get called automatically if the field storing this is marked #\[state\] and is
    /// inside a widget. This should get called before the widget receives events but after its
    /// parent widget receives its event. If the sync resulted in a state updating, the method
    /// will return true.
    #[allow(unused_variables)]
    fn sync(&mut self, env: &mut Environment) -> bool {
        false
    }
}
