use std::any::TypeId;
use std::pin::Pin;
use carbide_core::environment::Environment;
use carbide_core::plugin::Plugin;
use carbide_core::register;
use carbide_core::text::TextContextKey;
use crate::text_context::CosmicTextContext;

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        TextContextKey::insert(env, CosmicTextContext::new());
    })
}

register!(Plugin {
    dependencies: &[],
    registers: TypeId::of::<TextContextKey>(),
    register
});
