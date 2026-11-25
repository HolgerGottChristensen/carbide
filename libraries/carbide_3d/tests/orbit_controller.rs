//! Moving a view's camera with the orbit controller.

use carbide_3d::camera::{Camera, OrbitController};
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Node3dSequence, Mesh, Object, Scene3d};
use carbide_core::color::{BLUE, RED};
use carbide_core::draw::{Dimension, Position};
use carbide_core::math::{Point3, Vector3};
use carbide_core::widget::{HStack, WidgetExt};
use carbide_testing::Harness;

mod common;

/// A red cube at the origin, lit from above and to the left, seen by a perspective camera 3 units
/// in front of it, slightly above.
fn cube_scene() -> Scene3d<impl Node3dSequence> {
    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(RED));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);

    Scene3d::new((cube, light))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.5, 3.0), Point3::new(0.0, 0.0, 0.0)))
}

/// A view with an orbit controller, dragged to the right and then down. Pins down that dragging
/// inside the view turns the camera around the target: to the right, the scene turns right and the
/// cube's left side comes into view; down, its top does.
#[test]
fn dragging_orbits_the_camera() {
    let scene = cube_scene().controller(OrbitController::new()).border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("start");

    harness.drag(Position::new(200.0, 150.0), Position::new(300.0, 150.0), 5);
    harness.snapshot("dragged_right");

    harness.drag(Position::new(200.0, 150.0), Position::new(200.0, 225.0), 5);
    harness.snapshot("dragged_down");
}

/// A view with an orbit controller, scrolled right and down by the same distances
/// `dragging_orbits_the_camera` drags. Pins down that scrolling inside the view orbits the camera
/// as dragging the content would: the snapshot matches that test's `dragged_down`.
#[test]
fn scrolling_orbits_the_camera() {
    let scene = cube_scene().controller(OrbitController::new()).border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    // Lay the view out before sending input, as a window would have.
    harness.render();
    harness.mouse_move(Position::new(200.0, 150.0));
    harness.scroll(100.0, 0.0);
    harness.scroll(0.0, 75.0);
    harness.snapshot("scrolled");
}

/// A view with an orbit controller, pinched outwards. Pins down that a scale gesture inside the
/// view moves the camera towards the target, so the cube is drawn larger.
#[test]
fn pinching_zooms_the_camera() {
    let scene = cube_scene().controller(OrbitController::new()).border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    // Lay the view out before sending input, as a window would have.
    harness.render();
    harness.mouse_move(Position::new(200.0, 150.0));
    harness.pinch(0.5);
    harness.snapshot("pinched_out");
}

/// A view with an orbit controller and an orthographic camera, pinched outwards. Pins down that a
/// scale gesture zooms an orthographic camera by changing its projection's height, since moving it
/// closer would change nothing on screen: the cube is drawn larger.
#[test]
fn pinching_zooms_an_orthographic_camera() {
    let scene = cube_scene()
        .camera(Camera::orthographic(3.0).look_at(Point3::new(0.0, 0.5, 3.0), Point3::new(0.0, 0.0, 0.0)))
        .controller(OrbitController::new())
        .border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("start");

    harness.mouse_move(Position::new(200.0, 150.0));
    harness.pinch(0.5);
    harness.snapshot("pinched_out");
}

/// Two views of one scene, the left a clone with an orbit controller and the right without one,
/// both starting from the same camera. The left view is dragged. Pins down that a clone gets a
/// copy of the camera, not a share of it: the left view turns, and the right view stays where it
/// was.
#[test]
fn clones_have_their_own_camera() {
    let scene = cube_scene();

    let views = HStack::new((
        scene.clone().controller(OrbitController::new()).border().color(BLUE),
        scene.border().color(BLUE),
    ))
        .padding(20.0);

    let mut harness = Harness::new(views, Dimension::new(600.0, 300.0));
    // Lay the views out before sending input, as a window would have.
    harness.render();
    harness.drag(Position::new(150.0, 150.0), Position::new(250.0, 150.0), 5);
    harness.snapshot("left_dragged");
}
