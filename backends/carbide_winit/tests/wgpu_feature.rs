use carbide_core::environment::Environment;
use carbide_core::plugin::register_plugins;
use carbide_windowing::WindowRendererFactory;

#[test]
fn wgpu_feature_registers_window_renderer() {
    // Like an application, this test only names carbide_winit (a crate is linked only if it is
    // named). The `wgpu` feature should be enough to link carbide_wgpu and register its renderer,
    // with no `extern crate carbide_wgpu;` here.
    let _ = carbide_winit::WindowHandleKey;

    let mut env: Environment<'static> = Environment::new();
    futures::executor::block_on(register_plugins(&mut env));

    assert!(
        env.get::<WindowRendererFactory>().is_some(),
        "carbide_winit's `wgpu` feature should link carbide_wgpu, which registers a WindowRendererFactory"
    );
}
