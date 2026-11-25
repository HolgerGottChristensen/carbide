//! Sharing meshes between objects, and changing an object's mesh.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Mesh, Node3dExt, Object, Scene3d};
use carbide_core::color::{BLUE, GREEN, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Point3, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::WidgetExt;
use carbide_testing::Harness;

mod common;

fn camera() -> Camera {
    Camera::perspective().look_at(Point3::new(0.0, 1.0, 4.0), Point3::new(0.0, 0.0, 0.0))
}

/// Two objects drawing the same mesh, a red one on the left and a green one on the right. Pins
/// down that objects sharing a mesh's GPU copy each draw with their own transform and material.
#[test]
fn objects_sharing_a_mesh() {
    let ball = Mesh::icosphere(0.6, 2);

    let left = Object::new(ball.clone(), PbrMaterial::new().color(RED)).position(Vector3::new(-0.8, 0.0, 0.0));
    let right = Object::new(ball, PbrMaterial::new().color(GREEN)).position(Vector3::new(0.8, 0.0, 0.0));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(3.0);

    let scene = Scene3d::new((left, right, light)).camera(camera()).border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("two_balls");
}

/// An object whose mesh is a state, set from a cube to a sphere between the two snapshots. Pins
/// down that an object follows its mesh state after the scene is built.
#[test]
fn an_object_follows_its_mesh_state() {
    let mut mesh = LocalState::new(Mesh::cube(1.0));

    let object = Object::new(mesh.clone(), PbrMaterial::new().color(RED));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(3.0);

    let scene = Scene3d::new((object, light)).camera(camera()).border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("cube");

    *mesh.value_mut() = Mesh::icosphere(0.6, 3);
    harness.snapshot("sphere");
}
