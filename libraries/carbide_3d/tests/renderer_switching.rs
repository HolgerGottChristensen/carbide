//! Switching between a renderer that draws and one that draws nothing.

use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::render::{ContextFactory3d, NoopRenderContext3d, RenderContext3dId};
use carbide_3d::{Node3dExt, Object, Scene3d};
use carbide_core::color::{BLUE, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{EnvUpdatingNew2, WidgetExt};
use carbide_testing::Harness;
use carbide_wgpu_3d::WgpuRenderContext3d;

mod common;

fn wgpu() -> ContextFactory3d {
    ContextFactory3d { id: RenderContext3dId::of::<WgpuRenderContext3d>(), render_context: |_| Box::new(WgpuRenderContext3d::new()) }
}

fn none() -> ContextFactory3d {
    ContextFactory3d { id: RenderContext3dId::of::<NoopRenderContext3d>(), render_context: |_| Box::new(NoopRenderContext3d) }
}

/// A lit cube under a renderer factory held in a state: rendered with wgpu, then with the no-op
/// renderer, then with wgpu again. Pins down that switching to a renderer that draws nothing
/// leaves the view empty, not showing the last frame the previous renderer drew, and that
/// switching back draws the cube again.
#[test]
fn switching_to_no_renderer_empties_the_view() {
    let mut factory = LocalState::new(wgpu());

    let cube = Object::new(common::cube(1.0), PbrMaterial::new().color(RED))
        .position(Vector3::new(0.0, 0.0, 3.0))
        .rotation(Euler::new(Deg(25.0), Deg(35.0), Deg(0.0)));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, 1.5)).intensity(5.0);

    let scene = EnvUpdatingNew2::<_, ContextFactory3d, _>::new(
        factory.clone(),
        Scene3d::new((cube, light)).border().color(BLUE).padding(20.0),
    );

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("wgpu");

    *factory.value_mut() = none();
    harness.snapshot("none");

    *factory.value_mut() = wgpu();
    harness.snapshot("wgpu_again");
}
