use std::any::TypeId;
use std::pin::Pin;
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

/// Runs every registered plugin. A plugin only runs once every type in its `dependencies` has
/// been registered by an earlier plugin.
///
/// Panics if some plugins can never run, because a dependency is registered by no plugin or the
/// dependencies form a cycle.
pub async fn register_plugins(env: &mut Environment<'static>) {
    let mut pending: Vec<&Plugin> = inventory::iter::<Plugin>.into_iter().collect();
    let mut registered: Vec<TypeId> = Vec::with_capacity(pending.len());

    while !pending.is_empty() {
        let ready = pending.iter().position(|plugin| {
            plugin.dependencies.iter().all(|dependency| registered.contains(dependency))
        });

        let Some(index) = ready else {
            let unresolved: Vec<_> = pending.iter()
                .map(|plugin| (plugin.registers, plugin.dependencies))
                .collect();
            panic!(
                "plugins can not be registered, because their dependencies are never registered or depend on each other. Unresolved (registers, dependencies): {:?}",
                unresolved
            );
        };

        let plugin = pending.remove(index);
        (plugin.register)(env).await;
        registered.push(plugin.registers);
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
