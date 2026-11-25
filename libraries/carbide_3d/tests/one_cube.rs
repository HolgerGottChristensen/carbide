//! Rendering of a single lit object.

use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Node3dExt, Mesh, Object, Scene3d};
use carbide_core::color::{BLUE, GREEN, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{Widget, WidgetExt};
use carbide_testing::Harness;

mod common;

/// A cube with `material`, turned so three of its faces show, lit by one directional light from
/// above, the left and behind the camera. The camera is at the origin, looking along +Z, and the
/// cube is in front of it.
fn cube_scene(material: PbrMaterial) -> impl Widget {
    let cube = Object::new(Mesh::cube(1.0), material)
        .position(Vector3::new(0.0, 0.0, -3.0))
        .rotation(Euler::new(Deg(-25.0), Deg(-35.0), Deg(0.0)));

    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);

    Scene3d::new((cube, light))
        .border()
        .color(BLUE)
        .padding(20.0)
}

/// A red cube in [`cube_scene`]. Pins down that the wgpu backend renders a mesh with a PBR
/// material, a transform, a light and a perspective camera into the scene's layer: the three faces
/// are shaded differently, and the layer around the cube stays transparent.
#[test]
fn one_cube() {
    let mut harness = Harness::new(cube_scene(PbrMaterial::new().color(RED)), Dimension::new(400.0, 300.0));
    harness.snapshot("red");
}

/// A cube in [`cube_scene`] whose material colour is a state, changed from red to green between
/// the two snapshots. Pins down that a material follows its state after the scene is built: the
/// cube is drawn green, shaded as the red one was, with no other change to the scene.
#[test]
fn material_color_changed_between_renders() {
    let mut color = LocalState::new(RED);

    let mut harness = Harness::new(cube_scene(PbrMaterial::new().color(color.clone())), Dimension::new(400.0, 300.0));
    harness.snapshot("red");

    *color.value_mut() = GREEN;
    harness.snapshot("green");
}
