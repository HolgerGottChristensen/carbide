//! Material values: metallic, reflectance, clear coat, ambient occlusion, emission and vertex
//! colours.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::Vertex;
use carbide_3d::{Mesh, Node3dExt, Object, Scene3d};
use carbide_core::color::{rgba, rgba_bytes, BLACK, BLUE, RED, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::math::{Point3, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{Widget, WidgetExt};
use carbide_testing::Harness;

mod common;

fn framed(scene: impl Widget) -> impl Widget {
    scene.border().color(BLUE).padding(20.0)
}

/// The camera of most tests here: in front of the origin, slightly above.
fn camera() -> Camera {
    Camera::perspective().look_at(Point3::new(0.0, 0.8, 3.0), Point3::new(0.0, 0.0, 0.0))
}

fn light() -> impl carbide_3d::Node3d {
    DirectionalLight::new(Vector3::new(-0.5, -1.0, -0.8)).intensity(4.0)
}

/// The rendered picture's bytes, 200 by 200.
fn picture(scene: impl Widget) -> Vec<u8> {
    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    harness.pixels().as_raw().to_vec()
}

/// The pixel at the centre of a 200 by 200 picture.
fn centre(picture: &[u8]) -> [u8; 4] {
    let i = (100 * 200 + 100) * 4;
    [picture[i], picture[i + 1], picture[i + 2], picture[i + 3]]
}

fn difference(a: &[u8], b: &[u8]) -> u8 {
    a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0)
}

/// Five spheres from left to right: plastic, a reflective dielectric, gold-coloured metal, rough
/// metal, and rough plastic with a clear coat. Pins down that metallic, reflectance and clear coat
/// reach the shader: the metals take the colour into their highlights and have no diffuse light,
/// and the coated sphere's highlight is wider than the plastic's.
#[test]
fn metallic_reflectance_and_clearcoat() {
    let gold = rgba_bytes(255, 195, 86, 1.0);
    let sphere = |x: f32, material: PbrMaterial| Object::new(Mesh::icosphere(0.35, 4), material).position(Vector3::new(x, 0.0, 0.0));

    let scene = Scene3d::new((
        sphere(-1.6, PbrMaterial::new().color(RED).roughness(Some(0.3f32))),
        sphere(-0.8, PbrMaterial::new().color(RED).roughness(Some(0.3f32)).reflectance(1.0)),
        sphere(0.0, PbrMaterial::new().color(gold).roughness(Some(0.3f32)).metallic(1.0)),
        sphere(0.8, PbrMaterial::new().color(gold).roughness(Some(0.7f32)).metallic(1.0)),
        sphere(1.6, PbrMaterial::new().color(RED).roughness(Some(0.3f32)).clearcoat(1.0, 0.8)),
        light(),
    ))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.5, 3.2), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(framed(scene), Dimension::new(400.0, 200.0));
    harness.snapshot("spheres");
}

/// A sphere that gives off red light, in a scene with no lights at all. Pins down emission: the
/// sphere shows its colour though nothing lights it, and the strength scales it.
#[test]
fn an_emissive_sphere_glows_in_the_dark() {
    let glowing = |strength: f32| {
        let material = PbrMaterial::new().color(BLACK).emissive(RED).emissive_strength(strength);
        Scene3d::new(Object::new(Mesh::icosphere(1.0, 3), material)).ambient(WHITE, 0.0).camera(camera())
    };

    let full = centre(&picture(glowing(1.0)));
    let half = centre(&picture(glowing(0.3)));

    assert!(full[0] > 200 && full[1] < 30 && full[2] < 30, "the sphere should glow red, but is {full:?}");
    assert!(half[0] < full[0] && half[0] > 50, "a weaker glow should be dimmer: {half:?} against {full:?}");

    let mut harness = Harness::new(framed(glowing(1.0)), Dimension::new(300.0, 300.0));
    harness.snapshot("glowing");
}

/// A white sphere with an ambient occlusion of 0, lit by ambient light alone, and then by a
/// directional light alone. Pins down that ambient occlusion darkens only the ambient light: black
/// in the first, as with full occlusion in the second.
#[test]
fn ambient_occlusion_darkens_only_ambient_light() {
    let sphere = |occlusion: f32| Object::new(Mesh::icosphere(1.0, 3), PbrMaterial::new().color(WHITE).ambient_occlusion(occlusion));

    let ambient_occluded = centre(&picture(Scene3d::new(sphere(0.0)).ambient(WHITE, 1.0).camera(camera())));
    let ambient_open = centre(&picture(Scene3d::new(sphere(1.0)).ambient(WHITE, 1.0).camera(camera())));
    assert!(ambient_occluded[0] < 5 && ambient_open[0] > 100, "occluded {ambient_occluded:?}, open {ambient_open:?}");

    let lit_occluded = picture(Scene3d::new((sphere(0.0), light())).ambient(WHITE, 0.0).camera(camera()));
    let lit_open = picture(Scene3d::new((sphere(1.0), light())).ambient(WHITE, 0.0).camera(camera()));
    assert_eq!(difference(&lit_occluded, &lit_open), 0, "light from the scene's lights should be unaffected");
}

/// A cube whose vertices are coloured by their position: red along X, green along Y, blue along
/// Z. Pins down vertex colours: with `.vertex_colors()` the faces blend the corners' colours, and
/// without it the cube is plain white.
#[test]
fn vertex_colours() {
    let painted = || {
        let cube = Mesh::cube(1.0);
        let vertices = cube.vertices().iter().map(|vertex| {
            let p = vertex.position + Vector3::new(0.5, 0.5, 0.5);
            Vertex { color_0: rgba(p.x, p.y, p.z, 1.0), ..*vertex }
        }).collect();
        Mesh::new(vertices, cube.indices().to_vec())
    };
    let scene = |material: PbrMaterial| {
        Scene3d::new((Object::new(painted(), material).rotation(carbide_core::math::Euler::new(carbide_core::math::Deg(20.0f32), carbide_core::math::Deg(-35.0), carbide_core::math::Deg(0.0))), light()))
            .camera(camera())
    };

    let plain = picture(scene(PbrMaterial::new().color(WHITE)));
    let white_mesh = picture(Scene3d::new((Object::new(Mesh::cube(1.0), PbrMaterial::new().color(WHITE)).rotation(carbide_core::math::Euler::new(carbide_core::math::Deg(20.0f32), carbide_core::math::Deg(-35.0), carbide_core::math::Deg(0.0))), light())).camera(camera()));
    assert_eq!(difference(&plain, &white_mesh), 0, "without .vertex_colors(), vertex colours should be ignored");

    let mut harness = Harness::new(framed(scene(PbrMaterial::new().color(WHITE).vertex_colors())), Dimension::new(300.0, 300.0));
    harness.snapshot("painted_cube");
}

/// A sphere whose metallic value is a state, set from 0 to 1 between renders. Pins down that the
/// material's values are synced, not only its colour: the picture changes.
#[test]
fn material_values_follow_their_states() {
    let metallic = LocalState::new(0.0f32);
    let material = PbrMaterial::new().color(RED).roughness(Some(0.4f32)).metallic(metallic.clone());
    let scene = Scene3d::new((Object::new(Mesh::icosphere(1.0, 3), material), light())).camera(camera());

    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    let before = harness.pixels().as_raw().to_vec();

    metallic.clone().set_value(1.0);
    harness.render();
    let after = harness.pixels().as_raw().to_vec();

    assert!(difference(&before, &after) > 20, "making the sphere metal should change the picture");
}
