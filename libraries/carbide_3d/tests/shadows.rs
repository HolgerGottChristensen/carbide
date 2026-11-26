//! Shadows from directional and spot lights, and turning them on and off through the environment.

use carbide_3d::camera::Camera;
use carbide_3d::light::{DirectionalLight, SpotLight};
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Group, Mesh, Node3d, Node3dExt, Node3dSequence, Object, Scene3d, ShadowsExt};
use carbide_core::color::{BLUE, ORANGE, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Point3, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{Widget, WidgetExt};
use carbide_testing::Harness;

mod common;

fn size() -> Dimension {
    Dimension::new(400.0, 300.0)
}

/// The light of most tests here: 3 lux from the upper left, so shadows fall to the right.
fn sun() -> impl Node3d {
    DirectionalLight::new(Vector3::new(1.0, -1.5, -0.5)).intensity(3.0)
}

fn floor() -> Object {
    Object::new(Mesh::plane(4.0, 4.0), PbrMaterial::new().color(WHITE).roughness(Some(0.8f32)))
}

fn cube() -> impl Node3d {
    Object::new(Mesh::cube(0.5), PbrMaterial::new().color(ORANGE).roughness(Some(0.6f32)))
        .position(Vector3::new(0.0, 0.25, 0.0))
}

/// `nodes` seen from above and in front, with a little ambient light so shadows aren't black.
fn scene<N: Node3dSequence>(nodes: N) -> Scene3d<N> {
    Scene3d::new(nodes)
        .ambient(WHITE, 0.15)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.5, 2.5), Point3::new(0.0, 0.0, 0.0)))
}

fn framed(scene: impl Widget) -> impl Widget {
    scene.border().color(BLUE).padding(20.0)
}

/// The rendered picture's bytes, 200 by 200.
fn picture(scene: impl Widget) -> Vec<u8> {
    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    harness.pixels().as_raw().to_vec()
}

/// The sum of all channels of a picture: lower is darker.
fn brightness(picture: &[u8]) -> u64 {
    picture.iter().map(|byte| *byte as u64).sum()
}

/// The largest difference of any channel between two pictures.
fn difference(a: &[u8], b: &[u8]) -> u8 {
    a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0)
}

/// A cube on the floor, with shadows turned on for the whole scene. Pins down that a directional
/// light casts shadows where `CastsShadows` is set: the cube's shadow falls to its right, darker
/// than the floor around it and softened at the edges.
#[test]
fn a_directional_light_casts_a_shadow() {
    let mut harness = Harness::new(framed(scene((floor(), cube(), sun())).casts_shadows(true)), size());
    harness.snapshot("cube_shadow");
}

/// The scene of `a_directional_light_casts_a_shadow`, rendered with nothing set, and with
/// shadows turned off and on explicitly. Pins down that lights cast shadows unless told not to,
/// and that the shadow darkens the picture.
#[test]
fn lights_cast_shadows_by_default() {
    let unset = picture(scene((floor(), cube(), sun())));
    let off = picture(scene((floor(), cube(), sun())).casts_shadows(false));
    let on = picture(scene((floor(), cube(), sun())).casts_shadows(true));

    assert_eq!(difference(&unset, &on), 0, "nothing set should be the same as shadows on");
    assert!(brightness(&on) < brightness(&off), "a shadow should darken the picture");
}

/// Shadows turned off for the light alone, with `.casts_shadows(false)` on its node. Pins down
/// that the setting is read by the light from its node: with the only light not casting, the
/// picture is as with shadows off for the whole scene.
#[test]
fn a_light_opts_out_on_its_node() {
    let off_light = picture(scene((floor(), cube(), sun().casts_shadows(false))));
    let off_scene = picture(scene((floor(), cube(), sun())).casts_shadows(false));

    assert_eq!(difference(&off_light, &off_scene), 0);
}

/// Shadows on for the scene, but off for a group holding the cube. Pins down that the nearest
/// setting wins: the cube casts no shadow, so the picture is as without shadows.
#[test]
fn a_group_opts_out_of_casting() {
    let opted_out = picture(scene((floor(), Group::new(cube()).casts_shadows(false), sun())).casts_shadows(true));
    let off = picture(scene((floor(), cube(), sun())).casts_shadows(false));

    assert!(difference(&opted_out, &off) <= 1, "the cube should cast no shadow");
}

/// Shadows on for the scene, but the floor doesn't receive them. Pins down `ReceivesShadows`:
/// the floor shows no shadow, so the picture is as without shadows.
#[test]
fn the_floor_opts_out_of_receiving() {
    let opted_out = picture(scene((floor().receives_shadows(false), cube(), sun())).casts_shadows(true));
    let off = picture(scene((floor(), cube(), sun())).casts_shadows(false));

    assert!(difference(&opted_out, &off) <= 1, "the floor should show no shadow");
}

/// Shadows switched through a state. Pins down that the setting is reactive: the picture changes
/// when the state does, and goes back when it is reset.
#[test]
fn shadows_follow_a_state() {
    let casts = LocalState::new(false);
    let mut harness = Harness::new(scene((floor(), cube(), sun())).casts_shadows(casts.clone()), Dimension::new(200.0, 200.0));

    harness.render();
    let before = harness.pixels().as_raw().to_vec();

    casts.clone().set_value(true);
    harness.render();
    let during = harness.pixels().as_raw().to_vec();

    casts.clone().set_value(false);
    harness.render();
    let after = harness.pixels().as_raw().to_vec();

    assert!(brightness(&during) < brightness(&before), "turning shadows on should darken the picture");
    assert_eq!(difference(&before, &after), 0, "turning them off should restore the picture");
}

/// A spot light 2 metres above and to the left of the cube, aimed at it, casting shadows. Pins
/// down spot light shadows, from a perspective map: the cube's shadow stretches away from the
/// light inside the lit cone.
#[test]
fn a_spot_light_casts_a_shadow() {
    let spot = SpotLight::new(Deg(25.0), Deg(35.0))
        .intensity(12.0)
        .rotation(Euler::new(Deg(-60.0), Deg(-35.0), Deg(0.0)))
        .position(Vector3::new(-1.2, 2.0, 0.8))
        .casts_shadows(true);

    let mut harness = Harness::new(framed(scene((floor(), cube(), spot))), size());
    harness.snapshot("cube_spot_shadow");
}

/// Two views of a scene with shadows: one from the front, one from above. Pins down that each
/// view fits the directional map to its own camera, so both show the shadow.
#[test]
fn every_view_shows_the_shadow() {
    let front = scene((floor(), cube(), sun()));
    let above = front.clone()
        .camera(Camera::perspective().look_at(Point3::new(0.01, 4.0, 0.0), Point3::new(0.0, 0.0, 0.0)));

    let views = carbide_core::widget::HStack::new((front.frame(180.0, 180.0), above.frame(180.0, 180.0)))
        .spacing(10.0)
        .casts_shadows(true);

    let mut harness = Harness::new(framed(views), size());
    harness.snapshot("two_views");
}
