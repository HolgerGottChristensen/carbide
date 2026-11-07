use std::any::TypeId;
use std::pin::Pin;
use carbide_core::environment::Environment;
use carbide_core::plugin::Plugin;
use carbide_core::register;
use carbide_tesselation::TesselatorKey;
use crate::LyonTesselator;

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        TesselatorKey::insert(env, LyonTesselator::new());
    })
}

register!(Plugin {
    dependencies: &[],
    registers: TypeId::of::<TesselatorKey>(),
    register
});
