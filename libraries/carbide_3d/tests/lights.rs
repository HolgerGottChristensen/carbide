//! Point and spot lights.

use carbide_3d::camera::Camera;
use carbide_3d::light::{DirectionalLight, PointLight, SpotLight};
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Mesh, Node3d, Node3dExt, Node3dSequence, Object, Scene3d};
use carbide_core::color::{BLUE, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Point3, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{Widget, WidgetExt};
use carbide_testing::Harness;

mod common;

fn size() -> Dimension {
    Dimension::new(400.0, 300.0)
}

/// A white floor 4 by 4 metres, seen from above and in front, with no ambient light.
fn floor<L: Node3d>(light: L) -> Scene3d<impl Node3dSequence> {
    let floor = Object::new(Mesh::plane(4.0, 4.0), PbrMaterial::new().color(WHITE).roughness(Some(0.8f32)));

    Scene3d::new((floor, light))
        .ambient(WHITE, 0.0)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 3.0, 2.5), Point3::new(0.0, 0.0, 0.0)))
}

/// The pixel at the centre of `scene`, rendered 200 by 200: where the floor's centre is.
fn centre_pixel(scene: impl Widget) -> [u8; 4] {
    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    harness.pixels().get_pixel(100, 100).0
}

fn framed(scene: impl Widget) -> impl Widget {
    scene.border().color(BLUE).padding(20.0)
}

/// The floor lit by a point light of 4 candela 1 metre above its centre, and by a directional
/// light of 4 lux shining straight down. Pins down the units and the inverse square law: at the
/// centre, the two light the floor exactly alike.
#[test]
fn a_candela_at_a_metre_is_a_lux() {
    let point = centre_pixel(floor(PointLight::new().intensity(4.0).position(Vector3::new(0.0, 1.0, 0.0))));
    let directional = centre_pixel(floor(DirectionalLight::new(Vector3::new(0.0, -1.0, 0.0)).intensity(4.0)));

    assert!(point[0] > 100, "the floor's centre should be lit, but is {point:?}");

    let difference = point.iter().zip(directional.iter()).map(|(a, b)| (*a as i32 - *b as i32).abs()).max().unwrap();
    assert!(difference <= 1, "the point light gives {point:?}, the directional light {directional:?}");
}

/// The floor lit by a point light of 2 candela, 0.6 metres above its centre. Pins down that a
/// point light falls off with distance: brightest straight below it, fading towards the floor's
/// edges, lighting all around.
#[test]
fn a_point_light_falls_off_with_distance() {
    let light = PointLight::new().intensity(2.0).position(Vector3::new(0.0, 0.6, 0.0));

    let mut harness = Harness::new(framed(floor(light)), size());
    harness.snapshot("pool_of_light");
}

/// The point light of `a_point_light_falls_off_with_distance`, with a range of 1.2 metres. Pins
/// down that a range ends a light smoothly: it lights only a disc about a metre across, and the
/// rest of the floor is dark.
#[test]
fn a_range_ends_a_light() {
    let light = PointLight::new().intensity(2.0).range(1.2).position(Vector3::new(0.0, 0.6, 0.0));

    let mut harness = Harness::new(framed(floor(light)), size());
    harness.snapshot("disc");
}

/// A spot light 1.5 metres above the floor's centre, turned to shine straight down, with a cone
/// of 15° inside and 25° outside. Pins down that a spot light shines along its node's +Z, aimed
/// with `.rotation()`: a bright disc under it with a soft edge, and darkness outside the cone.
#[test]
fn a_spot_light_lights_a_cone() {
    let light = SpotLight::new(Deg(15.0), Deg(25.0))
        .intensity(12.0)
        .rotation(Euler::new(Deg(-90.0), Deg(0.0), Deg(0.0)))
        .position(Vector3::new(0.0, 1.5, 0.0));

    let mut harness = Harness::new(framed(floor(light)), size());
    harness.snapshot("cone");
}

/// The spot light of `a_spot_light_lights_a_cone`, its rotation a state, tilted from straight
/// down to 60° below the horizon towards the right between the two snapshots. Pins down that a
/// spot light turns with its transform: its pool of light moves right and stretches.
#[test]
fn a_spot_light_turns_with_its_rotation() {
    let mut rotation = LocalState::new(Euler::new(Deg(-90.0f32), Deg(0.0), Deg(0.0)));

    let light = SpotLight::new(Deg(15.0), Deg(25.0))
        .intensity(12.0)
        .rotation(rotation.clone())
        .position(Vector3::new(0.0, 1.5, 0.0));

    let mut harness = Harness::new(framed(floor(light)), size());
    harness.snapshot("down");

    // cgmath composes Euler angles as Rx · Ry · Rz: +Z turned 30° about Y, then 90° about X,
    // points right and 60° below the horizon.
    *rotation.value_mut() = Euler::new(Deg(-90.0), Deg(-30.0), Deg(0.0));
    harness.snapshot("tilted");
}
