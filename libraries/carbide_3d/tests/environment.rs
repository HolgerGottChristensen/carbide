//! A scene's ambient light, and how a view looks at its scene: background, exposure and tone
//! mapping.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::render::Tonemapping;
use carbide_3d::{Mesh, Node3dExt, Node3dSequence, Object, Scene3d};
use carbide_core::color::{rgba_bytes, BLUE, RED, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::math::{Deg, Euler, Point3, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{HStack, WidgetExt};
use carbide_testing::Harness;

mod common;

fn size() -> Dimension {
    Dimension::new(400.0, 300.0)
}

/// A red cube turned so three faces show, lit by one bright directional light.
fn lit_cube() -> Scene3d<impl Node3dSequence> {
    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(RED))
        .rotation(Euler::new(Deg(25.0), Deg(35.0), Deg(0.0)));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, 1.5)).intensity(5.0);

    Scene3d::new((cube, light))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, -3.0), Point3::new(0.0, 0.0, 0.0)))
}

/// An empty view whose background is the theme's system background, the colour the window is
/// filled with. Pins down that a background is converted as the 2D renderer converts colours:
/// the view can't be told apart from the window around it.
#[test]
fn a_background_matches_the_ui_around_it() {
    let scene = Scene3d::new(()).background(EnvironmentColor::SystemBackground).padding(20.0);

    let mut harness = Harness::new(scene, size());
    harness.render();
    let pixels = harness.pixels();

    let outside = *pixels.get_pixel(5, 5);
    let inside = *pixels.get_pixel(200, 150);
    assert_eq!(inside, outside, "the view's background differs from the window around it");
}

/// The lit cube in front of a dark blue background. Pins down that the background fills the view
/// behind what is drawn, and that the cube's edges blend into it.
#[test]
fn a_background_colour() {
    let scene = lit_cube().background(rgba_bytes(20, 30, 70, 1.0)).border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, size());
    harness.snapshot("dark_blue");
}

/// A white sphere with no directional light, lit only by a hemisphere: white from above and
/// orange-brown from below. Pins down that the hemisphere lights surfaces by the way they face:
/// the top is lit by the sky, the bottom by the ground, and the sides by a blend of both.
#[test]
fn a_hemisphere_lights_from_above_and_below() {
    let sphere = Object::new(Mesh::icosphere(1.0, 3), PbrMaterial::new().color(WHITE));

    let scene = Scene3d::new(sphere)
        .hemisphere(WHITE, rgba_bytes(160, 90, 40, 1.0), 0.8)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, -3.0), Point3::new(0.0, 0.0, 0.0)))
        .border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, size());
    harness.snapshot("sky_and_ground");
}

/// The lit cube with its view's exposure in a state, changed from 1 to 0.3 between the two
/// snapshots. Pins down that exposure scales the lit colours before tone mapping: the cube
/// darkens, its brightest face most.
#[test]
fn exposure_follows_its_state() {
    let mut exposure = LocalState::new(1.0f32);
    let scene = lit_cube().exposure(exposure.clone()).border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, size());
    harness.snapshot("one");

    *exposure.value_mut() = 0.3;
    harness.snapshot("darker");
}

/// Three views of the lit cube side by side, with PBR Neutral, ACES and no tone mapping. Pins down
/// that each view tone maps on its own: PBR Neutral lightens the clipping face towards white, ACES
/// adds contrast, and with none the brightest face clips to pure red.
#[test]
fn each_view_has_its_own_tonemapping() {
    let scene = lit_cube();

    let views = HStack::new((
        scene.clone().tonemapping(Tonemapping::PbrNeutral).border().color(BLUE),
        scene.clone().tonemapping(Tonemapping::Aces).border().color(BLUE),
        scene.tonemapping(Tonemapping::None).border().color(BLUE),
    ))
        .spacing(10.0)
        .padding(10.0);

    let mut harness = Harness::new(views, Dimension::new(600.0, 220.0));
    harness.snapshot("neutral_aces_none");
}

/// Two views of a sphere lit only by its ambient light, the ambient set through the second view
/// after cloning. Pins down that the ambient light belongs to the scene: setting it on one view
/// lights both.
#[test]
fn ambient_light_is_shared_by_every_view() {
    let sphere = Object::new(Mesh::icosphere(1.0, 3), PbrMaterial::new().color(WHITE));
    let scene = Scene3d::new(sphere)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, -3.0), Point3::new(0.0, 0.0, 0.0)));

    let first = scene.clone();
    let second = scene.ambient(rgba_bytes(80, 200, 120, 1.0), 0.8);

    let views = HStack::new((first.border().color(BLUE), second.border().color(BLUE)))
        .spacing(10.0)
        .padding(10.0);

    let mut harness = Harness::new(views, Dimension::new(500.0, 250.0));
    harness.snapshot("both_green");
}
