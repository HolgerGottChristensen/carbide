// Nothing below names a carbide_wgpu item, so without this the crate is not linked and its
// plugins never register. Applications have the same requirement.
extern crate carbide_wgpu;

use carbide_core::environment::Environment;
use carbide_core::plugin::register_plugins;
use carbide_windowing::WindowRendererFactory;

#[test]
fn plugin_registers_window_renderer_factory() {
    let mut env: Environment<'static> = Environment::new();
    futures::executor::block_on(register_plugins(&mut env));

    let factory = env.get::<WindowRendererFactory>()
        .expect("carbide_wgpu should register a WindowRendererFactory through its plugin");

    // The image context is what the windowing backend hands to every event and render pass.
    let _image_context = (factory.image_context)();
}
