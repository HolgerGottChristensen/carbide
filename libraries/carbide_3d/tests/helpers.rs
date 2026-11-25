//! Helpers for finding your way around a scene, and unlit materials.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{AxisHelper, Mesh, Object, Scene3d};
use carbide_core::color::{rgba_bytes, BLUE, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::math::{Point3, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::WidgetExt;
use carbide_testing::Harness;

mod common;

/// A white cube of half a metre at the origin with an axis helper a metre long, its being shown a
/// state, turned off between the two snapshots. Pins down that the axes are thin lines from the
/// origin, red along +X (right), green along +Y (up) and blue along +Z (towards the viewer),
/// coming out of the cube; and that turning them off leaves only the cube.
#[test]
fn axes_can_be_shown_and_hidden() {
    let mut shown = LocalState::new(true);

    let cube = Object::new(Mesh::cube(0.5), PbrMaterial::new().color(WHITE));
    let axes = AxisHelper::new(1.0).shown(shown.clone());
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(3.0);

    let scene = Scene3d::new((cube, axes, light))
        .camera(Camera::perspective().look_at(Point3::new(2.0, 1.5, 2.5), Point3::new(0.3, 0.3, 0.3)))
        .border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("shown");

    *shown.value_mut() = false;
    harness.snapshot("hidden");
}

/// An unlit orange cube facing the camera under a bright light, rendered 200 by 200. Pins down
/// that an unlit material shows its colour exactly: the pixel at the face's centre is the colour
/// given, unaffected by the light, exposure and tone mapping.
#[test]
fn an_unlit_material_keeps_its_exact_colour() {
    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(rgba_bytes(200, 100, 50, 1.0)).unlit());
    let light = DirectionalLight::new(Vector3::new(0.0, 0.0, -1.0)).intensity(20.0);

    let scene = Scene3d::new((cube, light))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, 3.0), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    let pixel = harness.pixels().get_pixel(100, 100).0;

    let difference = pixel.iter().zip([200u8, 100, 50, 255]).map(|(a, b)| (*a as i32 - b as i32).abs()).max().unwrap();
    assert!(difference <= 1, "the unlit face is {pixel:?}, not (200, 100, 50)");
}
