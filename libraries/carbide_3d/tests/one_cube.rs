//! Rendering of a single lit object.

use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::material::Material;
use carbide_3d::{Node3dExt, Mesh, Object, Scene3d};
use carbide_core::color::{BLUE, GREEN, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Vector3};
use carbide_core::lens;
use carbide_core::state::{IntoReadState, LocalState, Map1, State};
use carbide_core::widget::{Widget, WidgetExt};
use carbide_testing::Harness;

mod common;

/// A cube with `material`, turned so three of its faces show, lit by one directional light from
/// above, the left and behind the camera. The camera is at the origin, looking along +Z, and the
/// cube is in front of it.
fn cube_scene(material: impl IntoReadState<Material>) -> impl Widget {
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

/// A cube in [`cube_scene`] whose material is mapped from a colour state, changed from red to
/// green between the two snapshots. Pins down that an object follows its material's state after the
/// scene is built: the cube is drawn green, shaded as the red one was, with no other change to the
/// scene.
#[test]
fn material_color_changed_between_renders() {
    let mut color = LocalState::new(RED);
    let material = Map1::read_map(color.clone(), |color| PbrMaterial::new().color(*color));

    let mut harness = Harness::new(cube_scene(material), Dimension::new(400.0, 300.0));
    harness.snapshot("red");

    *color.value_mut() = GREEN;
    harness.snapshot("green");
}

/// A cube in [`cube_scene`] whose material is a state of its own, edited through a field state
/// from `lens!`, as a slider would: the cube changes, and setting the field back restores it.
#[test]
fn material_edited_through_a_lens() {
    let material = LocalState::new(PbrMaterial::new().color(RED));
    let mut harness = Harness::new(cube_scene(material.clone()), Dimension::new(400.0, 300.0));
    harness.render();
    let before = harness.pixels().as_raw().to_vec();

    let mut metallic = lens!(material.metallic);
    *metallic.value_mut() = 1.0;
    harness.render();
    let metal = harness.pixels().as_raw().to_vec();
    let changed = before.iter().zip(&metal).filter(|(a, b)| a != b).count();
    assert!(changed > 1_000, "a metallic cube changes only {changed} bytes");

    *metallic.value_mut() = 0.0;
    harness.render();
    assert!(harness.pixels().as_raw().to_vec() == before, "setting the field back doesn't restore the cube");
}
