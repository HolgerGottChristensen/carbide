//! Switching the 3D renderer while a scene is shown.

use std::sync::Mutex;

use carbide_3d::camera::CameraSpec;
use carbide_3d::material::Material;
use carbide_3d::render::{ContextFactory3d, InnerRenderContext3d, RenderContext3dId};
use carbide_3d::{InnerImageContext3d, Mesh, NoopImageContext3d, Scene3d};
use carbide_core::color::Color;
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_core::math::{Matrix4, Vector3};
use carbide_core::render::Layer;
use carbide_core::state::{LocalState, State};
use carbide_core::widget::EnvUpdatingNew2;
use carbide_testing::Harness;

/// What the test backends did, in order: `"create A"`, `"render B"` and so on.
static EVENTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn take_events() -> Vec<String> {
    std::mem::take(&mut *EVENTS.lock().unwrap())
}

/// A render context that draws nothing and records when it is created and rendered.
#[derive(Debug, Clone)]
struct RecordingContext {
    name: &'static str,
    image_context: NoopImageContext3d,
}

impl RecordingContext {
    fn create(name: &'static str) -> Box<dyn InnerRenderContext3d> {
        EVENTS.lock().unwrap().push(format!("create {name}"));
        Box::new(RecordingContext { name, image_context: NoopImageContext3d })
    }
}

impl InnerRenderContext3d for RecordingContext {
    fn start(&mut self) {}

    fn render(&mut self, _layer: Layer, _camera_spec: CameraSpec, _env: &mut Environment) {
        EVENTS.lock().unwrap().push(format!("render {}", self.name));
    }

    fn mesh(&mut self, _mesh: &Mesh) {}

    fn material(&mut self, _material: &Material, _env: &mut Environment) {}

    fn pop_material(&mut self) {}

    fn transform(&mut self, _transform: &Matrix4<f32>) {}

    fn pop_transform(&mut self) {}

    fn directional(&mut self, _color: Color, _intensity: f32, _direction: Vector3<f32>) {}

    fn image_context(&mut self) -> &mut dyn InnerImageContext3d {
        &mut self.image_context
    }
}

struct BackendA;
struct BackendB;

fn factory_a() -> ContextFactory3d {
    ContextFactory3d { id: RenderContext3dId::of::<BackendA>(), render_context: |_| RecordingContext::create("A") }
}

fn factory_b() -> ContextFactory3d {
    ContextFactory3d { id: RenderContext3dId::of::<BackendB>(), render_context: |_| RecordingContext::create("B") }
}

/// A scene under a 3D renderer factory held in a state, which is changed between renders. Pins
/// down that a `Scene3d` creates its render context from the factory in the environment, keeps it
/// while the factory stays the same, and recreates it as soon as the factory changes, also when
/// switching back to a renderer it used before.
#[test]
fn switching_the_factory_recreates_the_render_context() {
    let mut factory = LocalState::new(factory_a());

    let scene = EnvUpdatingNew2::<_, ContextFactory3d, _>::new(
        factory.clone(),
        Scene3d::new(()),
    );

    let mut harness = Harness::new(scene, Dimension::new(200.0, 100.0));
    take_events();

    harness.render();
    assert_eq!(take_events(), ["create A", "render A"], "the first render creates a context from the factory");

    harness.render();
    assert_eq!(take_events(), ["render A"], "the same factory keeps the context");

    *factory.value_mut() = factory_b();
    harness.render();
    assert_eq!(take_events(), ["create B", "render B"], "a new factory replaces the context");

    *factory.value_mut() = factory_a();
    harness.render();
    assert_eq!(take_events(), ["create A", "render A"], "switching back creates a fresh context");
}
