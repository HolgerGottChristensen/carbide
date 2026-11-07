use std::any::TypeId;
use std::pin::Pin;
use futures::future::BoxFuture;
use crate::environment::Environment;

inventory::collect!(Plugin);

pub struct Plugin {
    // TODO: What if we want multiple types to exist. What if we want one type or another to exist.
    /// A set of types that should be registered previously to this plugin being run.
    /// This is used to topo-sort the plugins, to ensure plugins that depend on each other
    /// are registered in the correct order.
    pub dependencies: &'static [TypeId],

    pub registers: TypeId,

    // TODO: How do we guarantee a plugin registers some types. How can we know what was registered. Maybe comp time reflection
    // TODO: Relax the constraint that this must be static
    /// A function that is run when the initial application environment is being run.
    pub register: for<'a> fn(&'a mut Environment<'static>) -> Pin<Box<dyn Future<Output = ()> + 'a>>,
}

#[doc(hidden)]
pub use inventory::submit;

pub async fn register_plugins(env: &mut Environment<'static>) {

    for plugin in inventory::iter::<Plugin> {
        (plugin.register)(env).await
    }

    println!("{:#?}", env);
}

#[macro_export]
macro_rules! register {
    ($($value:tt)*) => {
        $crate::plugin::submit! {
            { $($value)* }
        }
    };
}
