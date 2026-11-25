use std::any::TypeId;
use std::pin::Pin;
use carbide::environment::Environment;
use carbide::register;
use carbide_derive::StateValue;
use crate::environment::{EnvironmentKey, Shared};
use crate::plugin::Plugin;

/// The direction used to layout widgets. It interacts with Alignment,
/// and changes its Leading and Trailing between left and right.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, StateValue)]
pub enum LayoutDirection {
    /// Leading ->->-> Trailing
    #[default]
    LeftToRight,
    /// Trailing <-<-<- Leading
    RightToLeft
}

impl EnvironmentKey for LayoutDirection {
    type Value = LayoutDirection;
    type Access = Shared;
}


// TODO: For now we register as a plugin with a fixed value. In the future we might place the value on the application and let it or a manager control the value.
fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        env.insert::<LayoutDirection>(&LayoutDirection::LeftToRight);
    })
}

register!(Plugin {
    dependencies: &[],
    registers: TypeId::of::<LayoutDirection>(),
    register
});