use carbide_core::draw::{Dimension, NOOPImageContext};
use carbide_core::environment::Environment;
use carbide_core::widget::Empty;
use carbide_windowing::{PhysicalSize, WindowRendererFactory};

use support::{environment_with, events, factory_a, factory_a_variant, factory_b, renderer_with, take_events};

#[test]
fn keeps_backend_while_factory_is_unchanged() {
    let mut env = environment_with(factory_a());
    let mut renderer = renderer_with(factory_a(), &mut env);

    renderer.resize(PhysicalSize::new(20, 20), 1.0, &mut env);

    assert_eq!(events(), vec!["resize a 20x20"]);
}

#[test]
fn recreates_backend_before_render_when_factory_changes() {
    let mut env = environment_with(factory_a());
    let mut renderer = renderer_with(factory_a(), &mut env);

    let b = factory_b();
    env.with::<WindowRendererFactory>(&b, |env| {
        renderer.render(&mut Empty::new(), Dimension::new(10.0, 10.0), 1.0, &mut NOOPImageContext, env).unwrap();
    });

    // The old backend is dropped before the new one is created.
    assert_eq!(events(), vec!["drop a", "create b 10x10", "render b"]);
    assert_eq!(renderer.renderer_id(), factory_b().id);
}

#[test]
fn recreates_backend_at_new_size_when_factory_changes_during_resize() {
    let mut env = environment_with(factory_a());
    let mut renderer = renderer_with(factory_a(), &mut env);

    let b = factory_b();
    env.with::<WindowRendererFactory>(&b, |env| {
        renderer.resize(PhysicalSize::new(30, 40), 2.0, env);
    });

    // Created at the new size, so it isn't resized afterwards.
    assert_eq!(events(), vec!["drop a", "create b 30x40"]);
}

#[test]
fn recreates_backend_before_capture_when_factory_changes() {
    let mut env = environment_with(factory_a());
    let mut renderer = renderer_with(factory_a(), &mut env);

    let b = factory_b();
    env.with::<WindowRendererFactory>(&b, |env| {
        let _ = renderer.capture(env);
    });

    assert_eq!(events(), vec!["drop a", "create b 10x10", "capture b"]);
}

#[test]
fn recreated_backend_uses_the_latest_size() {
    let mut env = environment_with(factory_a());
    let mut renderer = renderer_with(factory_a(), &mut env);
    renderer.resize(PhysicalSize::new(50, 60), 1.0, &mut env);
    take_events();

    let b = factory_b();
    env.with::<WindowRendererFactory>(&b, |env| {
        let _ = renderer.capture(env);
    });

    assert_eq!(events(), vec!["drop a", "create b 50x60", "capture b"]);
}

#[test]
fn factories_with_the_same_id_are_the_same_factory() {
    let mut env = environment_with(factory_a_variant());
    let mut renderer = renderer_with(factory_a(), &mut env);

    renderer.resize(PhysicalSize::new(20, 20), 1.0, &mut env);

    assert_eq!(events(), vec!["resize a 20x20"]);
}

#[test]
fn keeps_backend_when_environment_has_no_factory() {
    let mut env: Environment<'static> = Environment::new();
    let mut renderer = renderer_with(factory_a(), &mut env);

    renderer.resize(PhysicalSize::new(20, 20), 1.0, &mut env);

    assert_eq!(events(), vec!["resize a 20x20"]);
    assert_eq!(renderer.renderer_id(), factory_a().id);
}

// ===============================
//  Utility
// ===============================

mod support {
    use std::cell::RefCell;
    use std::error::Error;
    use std::sync::Arc;

    use carbide_core::draw::{Dimension, ImageContext, NOOPImageContext, Scalar};
    use carbide_core::environment::Environment;
    use carbide_core::widget::AnyWidget;
    use carbide_windowing::{PhysicalSize, SurfaceTarget, WindowRenderError, WindowRenderer, WindowRendererBackend, WindowRendererFactory, WindowRendererId};
    use raw_window_handle_06::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

    thread_local! {
        /// What the fake backends did, in order. Each test runs on its own thread.
        static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    fn log(event: String) {
        EVENTS.with(|events| events.borrow_mut().push(event));
    }

    pub fn events() -> Vec<String> {
        EVENTS.with(|events| events.borrow().clone())
    }

    pub fn take_events() -> Vec<String> {
        EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
    }

    /// A renderer for a 10x10 window, created by `factory`, with the creation already taken from
    /// the event log.
    pub fn renderer_with(factory: WindowRendererFactory, env: &mut Environment) -> WindowRenderer {
        let renderer = WindowRenderer::new(factory, target(), PhysicalSize::new(10, 10), 1.0, env);
        take_events();
        renderer
    }

    pub fn environment_with(factory: WindowRendererFactory) -> Environment<'static> {
        let mut env: Environment<'static> = Environment::new();
        env.insert::<WindowRendererFactory>(Box::leak(Box::new(factory)));
        env
    }

    /// A window without real handles. The fake backends never look at it.
    struct FakeTarget;

    impl HasWindowHandle for FakeTarget {
        fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
            Err(HandleError::Unavailable)
        }
    }

    impl HasDisplayHandle for FakeTarget {
        fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
            Err(HandleError::Unavailable)
        }
    }

    fn target() -> Arc<dyn SurfaceTarget> {
        Arc::new(FakeTarget)
    }

    struct FakeBackend {
        name: &'static str,
    }

    impl WindowRendererBackend for FakeBackend {
        fn resize(&mut self, size: PhysicalSize, _scale_factor: Scalar, _env: &mut Environment) {
            log(format!("resize {} {}x{}", self.name, size.width, size.height));
        }

        fn render(&mut self, _root: &mut dyn AnyWidget, _dimension: Dimension, _scale_factor: Scalar, _image: &mut dyn ImageContext, _env: &mut Environment) -> Result<(), WindowRenderError> {
            log(format!("render {}", self.name));
            Ok(())
        }

        fn capture(&mut self, _env: &mut Environment) -> Result<image::RgbaImage, Box<dyn Error>> {
            log(format!("capture {}", self.name));
            Err("fake backends can't capture".into())
        }
    }

    impl Drop for FakeBackend {
        fn drop(&mut self) {
            log(format!("drop {}", self.name));
        }
    }

    struct BackendA;
    struct BackendB;

    fn create(name: &'static str, size: PhysicalSize) -> Box<dyn WindowRendererBackend> {
        log(format!("create {} {}x{}", name, size.width, size.height));
        Box::new(FakeBackend { name })
    }

    fn create_a(_target: Arc<dyn SurfaceTarget>, size: PhysicalSize, _scale_factor: Scalar, _env: &mut Environment) -> Box<dyn WindowRendererBackend> {
        create("a", size)
    }

    fn create_a_variant(_target: Arc<dyn SurfaceTarget>, size: PhysicalSize, _scale_factor: Scalar, _env: &mut Environment) -> Box<dyn WindowRendererBackend> {
        create("a variant", size)
    }

    fn create_b(_target: Arc<dyn SurfaceTarget>, size: PhysicalSize, _scale_factor: Scalar, _env: &mut Environment) -> Box<dyn WindowRendererBackend> {
        create("b", size)
    }

    fn image_context() -> Box<dyn ImageContext> {
        Box::new(NOOPImageContext)
    }

    pub fn factory_a() -> WindowRendererFactory {
        WindowRendererFactory { id: WindowRendererId::of::<BackendA>(), create: create_a, image_context }
    }

    /// Same id as `factory_a`, but a different create function.
    pub fn factory_a_variant() -> WindowRendererFactory {
        WindowRendererFactory { id: WindowRendererId::of::<BackendA>(), create: create_a_variant, image_context }
    }

    pub fn factory_b() -> WindowRendererFactory {
        WindowRendererFactory { id: WindowRendererId::of::<BackendB>(), create: create_b, image_context }
    }
}
