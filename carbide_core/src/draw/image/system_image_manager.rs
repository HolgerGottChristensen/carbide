use std::fmt::Debug;
use std::rc::Rc;
use carbide::environment::EnvironmentKey;
use crate::environment::{Environment, Shared};

/// Provides the SVG data of system images, looked up by name. Registered into the environment
/// through the [SystemImageManager] key.
pub trait SystemImageProvider: Debug + 'static {
    /// The SVG data of the system image with the given name, or `None` if the provider does not
    /// have it.
    fn image(&self, name: &str) -> Option<&'static [u8]>;
}

/// The key used to look up the [SystemImageProvider] from the environment.
///
/// The value is reference counted, so a provider can keep a handle to the provider it replaces,
/// for example to fall back to it.
#[derive(Copy, Clone, Debug)]
pub struct SystemImageManager;

impl EnvironmentKey for SystemImageManager {
    type Value = Rc<dyn SystemImageProvider>;
    type Access = Shared;
}

impl SystemImageManager {
    /// Insert the provider into the environment, making it the provider retrieved by
    /// [SystemImageManager::get].
    pub fn insert(env: &mut Environment<'static>, provider: impl SystemImageProvider) {
        let value: Rc<dyn SystemImageProvider> = Rc::new(provider);
        env.insert::<SystemImageManager>(Box::leak(Box::new(value)));
    }

    /// Get a handle to the provider registered in the environment, if any.
    pub fn get(env: &Environment) -> Option<Rc<dyn SystemImageProvider>> {
        env.get::<SystemImageManager>().cloned()
    }
}
