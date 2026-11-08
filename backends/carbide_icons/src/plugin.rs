use std::any::TypeId;
use std::pin::Pin;
use std::rc::Rc;
use carbide_core::draw::{SystemImageManager, SystemImageProvider};
use carbide_core::environment::Environment;
use carbide_core::plugin::Plugin;
use carbide_core::register;
use crate::SYSTEM_IMAGE_MANAGER;

/// Provides the icons from this crate. Names it does not have are looked up in the provider that
/// was registered before it, if any.
#[derive(Debug)]
struct IconsProvider {
    fallback: Option<Rc<dyn SystemImageProvider>>,
}

impl SystemImageProvider for IconsProvider {
    fn image(&self, name: &str) -> Option<&'static [u8]> {
        SYSTEM_IMAGE_MANAGER(name)
            .or_else(|| self.fallback.as_ref().and_then(|fallback| fallback.image(name)))
    }
}

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        let fallback = SystemImageManager::get(env);
        SystemImageManager::insert(env, IconsProvider { fallback });
    })
}

register!(Plugin {
    dependencies: &[],
    registers: TypeId::of::<SystemImageManager>(),
    register
});
