//! The built-in shapes and OBJ loading.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{AnyNode3d, Group, Mesh, Node3dExt, Object, Scene3d};
use carbide_core::color::{BLUE, GREEN, ORANGE, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Point3, Vector3};
use carbide_core::widget::WidgetExt;
use carbide_testing::Harness;

mod common;

/// A sphere and a cube standing on a plane, seen from in front and above, lit from above and to
/// the left. Pins down the built-in shapes: the sphere is round, made of small flat faces, lit on
/// its upper left; the cube shows three faces; and the plane faces up, lit, under both.
#[test]
fn shapes_on_a_plane() {
    let floor = Object::new(Mesh::plane(4.0, 3.0), PbrMaterial::new().color(GREEN));
    let sphere = Object::new(Mesh::icosphere(0.6, 2), PbrMaterial::new().color(RED))
        .position(Vector3::new(-0.8, 0.6, 0.0));
    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(ORANGE))
        .position(Vector3::new(0.8, 0.5, 0.0));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.0)).intensity(5.0);

    let scene = Scene3d::new((floor, sphere, cube, light))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.5, 4.0), Point3::new(0.0, 0.3, 0.0)))
        .border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("shapes");
}

/// The render ball, loaded from its OBJ file with `Mesh::load_obj`. Pins down
/// that loading an OBJ file gives meshes that render the right way out, with the file's normals.
#[test]
fn an_obj_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/ressources/renderball.obj");
    let meshes = Mesh::load_obj(path).expect("the render ball to load");
    assert!(!meshes.is_empty());

    let parts = meshes.into_iter()
        .map(|mesh| Object::new(mesh, PbrMaterial::new().color(RED)).boxed())
        .collect::<Vec<Box<dyn AnyNode3d>>>();

    // The render ball is 1 unit across, standing on the origin.
    let ball = Group::new(parts);
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.0)).intensity(5.0);

    let scene = Scene3d::new((ball, light))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 1.2, 1.8), Point3::new(0.0, 0.5, 0.0)))
        .border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("render_ball");
}
