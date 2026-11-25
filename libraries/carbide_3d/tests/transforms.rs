//! Moving, turning and scaling nodes with the transform modifiers and groups.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Group, Node3dExt, Mesh, Object, Scene3d};
use carbide_core::color::{BLUE, GREEN, ORANGE, RED};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Point3, Vector3};
use carbide_core::state::{AnimatedState, LocalState, Map1, State};
use carbide_core::time::Duration;
use carbide_core::widget::{Widget, WidgetExt};
use carbide_testing::Harness;

mod common;

fn size() -> Dimension {
    Dimension::new(400.0, 300.0)
}

/// A camera 4 units in front of the origin, slightly above, looking at it.
fn camera() -> Camera {
    Camera::perspective().look_at(Point3::new(0.0, 1.0, 4.0), Point3::new(0.0, 0.0, 0.0))
}

fn framed(scene: impl Widget) -> impl Widget {
    scene.border().color(BLUE).padding(20.0)
}

/// Two cubes side by side in a group, the group turned 45° around the vertical axis. Pins down
/// that a group's nodes move together: the pair turns as one, around the group's origin between
/// them, so the right cube comes forward and the left one goes back.
#[test]
fn a_rotated_group_turns_its_nodes_together() {
    let pair = Group::new((
        Object::new(Mesh::cube(0.8), PbrMaterial::new().color(RED)).position(Vector3::new(-0.8, 0.0, 0.0)),
        Object::new(Mesh::cube(0.8), PbrMaterial::new().color(GREEN)).position(Vector3::new(0.8, 0.0, 0.0)),
    ))
        .rotation(Euler::new(Deg(0.0), Deg(-45.0), Deg(0.0)));

    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);

    let mut harness = Harness::new(framed(Scene3d::new((pair, light)).camera(camera())), size());
    harness.snapshot("turned");
}

/// Two cubes turned 45° around the axis towards the camera: the left around its centre, the
/// default pivot, and the right around its top right front corner. Pins down that a pivot is the
/// point a node turns around: the left cube turns in place, and the right one swings around its
/// corner, which stays where it was.
#[test]
fn a_pivot_is_what_a_node_turns_around() {
    let turn = Euler::new(Deg(0.0), Deg(0.0), Deg(45.0));

    let centre = Object::new(Mesh::cube(0.8), PbrMaterial::new().color(RED))
        .rotation(turn)
        .position(Vector3::new(-1.0, 0.0, 0.0));
    let corner = Object::new(Mesh::cube(0.8), PbrMaterial::new().color(ORANGE))
        .pivot(Vector3::new(0.4, 0.4, 0.4))
        .rotation(turn)
        .position(Vector3::new(1.0, 0.0, 0.0));

    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);

    let mut harness = Harness::new(framed(Scene3d::new((centre, corner, light)).camera(camera())), size());
    harness.snapshot("turned");
}

/// A cube whose rotation is a state of Euler angles, changed from none to 45° around the vertical
/// axis between the two snapshots. Pins down that a rotation follows its state, given as Euler
/// angles, after the scene is built.
#[test]
fn a_rotation_follows_its_state() {
    let mut rotation = LocalState::new(Euler::new(Deg(0.0f32), Deg(0.0), Deg(0.0)));

    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(RED)).rotation(rotation.clone());
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);

    let mut harness = Harness::new(framed(Scene3d::new((cube, light)).camera(camera())), size());
    harness.snapshot("still");

    *rotation.value_mut() = Euler::new(Deg(0.0), Deg(-45.0), Deg(0.0));
    harness.snapshot("turned");
}

/// The same cube as `a_rotation_follows_its_state`, turned by an animation: a full turn every
/// eight seconds, rendered at its start and after one second of the harness's simulated clock.
/// Pins down that an animation starts at its first render and follows the harness's clock
/// exactly: the second snapshot is that test's 45° `turned`, pixel for pixel.
#[test]
fn an_animated_rotation_follows_the_clock() {
    let angle = AnimatedState::linear().duration(Duration::from_secs(8)).repeat().range(0.0, 360.0);
    let rotation = Map1::read_map(angle, |angle: &f64| Euler::new(Deg(0.0), Deg(-*angle as f32), Deg(0.0)));

    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(RED)).rotation(rotation);
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);

    let mut harness = Harness::new(framed(Scene3d::new((cube, light)).camera(camera())), size());
    harness.snapshot("start");

    harness.advance_time(Duration::from_secs(1));
    harness.snapshot("after_a_second");
}

/// A cube facing the camera, and a light in a group of its own shining along +Z, away from the
/// camera, onto the cube's front. The group's rotation is a state, turned 180° around the vertical
/// axis between the two snapshots. Pins down that a light turns with the group it is in: at first
/// the front is lit, and once the group is turned, the light shines towards the camera and the
/// front is dark.
#[test]
fn a_light_turns_with_its_group() {
    let mut rotation = LocalState::new(Euler::new(Deg(0.0f32), Deg(0.0), Deg(0.0)));

    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(RED));
    let light = Group::new(DirectionalLight::new(Vector3::new(0.0, 0.0, -1.0)).intensity(5.0))
        .rotation(rotation.clone());

    let camera = Camera::perspective().look_at(Point3::new(0.0, 0.0, 3.0), Point3::new(0.0, 0.0, 0.0));
    let mut harness = Harness::new(framed(Scene3d::new((cube, light)).camera(camera)), size());
    harness.snapshot("front_lit");

    *rotation.value_mut() = Euler::new(Deg(0.0), Deg(-180.0), Deg(0.0));
    harness.snapshot("front_dark");
}
