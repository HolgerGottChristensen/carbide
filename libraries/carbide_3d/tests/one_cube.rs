//! Rendering of a single lit object.

use carbide_3d::camera::OrbitCamera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Object, Scene3d};
use carbide_core::color::{BLUE, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Matrix4, Vector3};
use carbide_core::widget::WidgetExt;
use carbide_testing::Harness;

mod common;

/// A red cube, turned so three of its faces show, lit by one directional light from above, the
/// left and behind the camera. The camera is at the origin, looking along +Z, and the cube is in
/// front of it. Pins down that the wgpu backend renders a mesh with a PBR material, a transform, a
/// light and a perspective camera into the scene's layer: the three faces are shaded differently,
/// and the layer around the cube stays transparent.
#[test]
fn one_cube() {
    let mut cube = Object::new(common::cube(1.0), PbrMaterial::new().color(RED));
    cube.transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, 3.0))
        * Matrix4::from(Euler::new(Deg(25.0), Deg(35.0), Deg(0.0)));

    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, 1.5)).intensity(5.0);

    let scene = Scene3d::new((cube, light), OrbitCamera::new())
        .border()
        .color(BLUE)
        .padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("red");
}
