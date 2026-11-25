//! Several views of one scene.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Node3dExt, Mesh, Object, Scene3d};
use carbide_core::color::{BLUE, GREEN, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Point3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{HStack, WidgetExt};
use carbide_testing::Harness;

mod common;

/// A perspective view of a cube from above and to the side, and a clone of it with a front
/// orthographic camera, side by side. The cube's material colour is a state, changed from red to
/// green between the two snapshots. Pins down that a clone of a `Scene3d` shows the same nodes
/// through its own camera, in its own layer, and that both views follow a change to the shared
/// nodes.
#[test]
fn two_views_of_one_scene() {
    let mut color = LocalState::new(RED);

    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(color.clone()))
        .rotation(Euler::new(Deg(0.0), Deg(30.0), Deg(0.0)));
    let light = DirectionalLight::new(carbide_core::math::Vector3::new(1.0, -2.0, 1.5)).intensity(5.0);

    let scene = Scene3d::new((cube, light))
        .camera(Camera::perspective().look_at(Point3::new(-2.0, 2.0, -3.0), Point3::new(0.0, 0.0, 0.0)));

    let views = HStack::new((
        scene.clone().border().color(BLUE),
        scene.camera(Camera::orthographic(2.5).look_at(Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 0.0, 0.0)))
            .border().color(BLUE),
    ))
        .padding(20.0);

    let mut harness = Harness::new(views, Dimension::new(600.0, 300.0));
    harness.snapshot("red");

    *color.value_mut() = GREEN;
    harness.snapshot("green");
}
