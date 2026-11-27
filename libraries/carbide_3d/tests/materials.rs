//! Material values: metallic, reflectance, clear coat, ambient occlusion, emission and vertex
//! colours.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::Vertex;
use carbide_3d::{Mesh, Node3dExt, Object, Scene3d};
use carbide_core::color::{rgba, rgba_bytes, BLACK, BLUE, ORANGE, RED, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::math::{Deg, Euler, Point3, Vector3};
use carbide_core::lens;
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

/// A sphere whose material is a state, its metallic value set from 0 to 1 between renders through
/// the field's state (`lens!`). Pins down that the object follows its material's values, not only
/// its colour: the picture changes.
#[test]
fn material_values_follow_their_states() {
    let material = LocalState::new(PbrMaterial::new().color(RED).roughness(0.4).metallic(0.0));
    let metallic = lens!(material.metallic);
    let scene = Scene3d::new((Object::new(Mesh::icosphere(1.0, 3), material.clone()), light())).camera(camera());

    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    let before = harness.pixels().as_raw().to_vec();

    metallic.clone().set_value(1.0);
    harness.render();
    let after = harness.pixels().as_raw().to_vec();

    assert!(difference(&before, &after) > 20, "making the sphere metal should change the picture");
}

/// The centre of a white plane, facing up, seen from straight below (`from_below`) or above, lit
/// only by a light travelling along `light`, with its material single- or double-sided.
fn plane_centre(double_sided: bool, from_below: bool, light: Vector3<f32>) -> u32 {
    let plane = Object::new(Mesh::plane(4.0, 4.0), PbrMaterial::new().color(WHITE).roughness(Some(1.0f32)).double_sided(double_sided));
    let eye = if from_below { Point3::new(0.0, -3.0, 0.001) } else { Point3::new(0.0, 3.0, 0.001) };
    let scene = Scene3d::new((plane, DirectionalLight::new(light).intensity(3.0)))
        .ambient(WHITE, 0.0)
        .camera(Camera::perspective().look_at(eye, Point3::new(0.0, 0.0, 0.0)));
    let [r, g, b, _] = centre(&picture(scene));
    r as u32 + g as u32 + b as u32
}

/// A plane's back is skipped unless its material is double-sided. Then it is lit as a surface
/// facing the other way: by a light from below, not from above, and as brightly as its front is
/// by a light from above.
#[test]
fn a_double_sided_material_shows_and_lights_its_back() {
    let upwards = Vector3::new(0.0, 1.0, 0.0);
    let downwards = Vector3::new(0.0, -1.0, 0.0);

    let background = {
        let scene = Scene3d::new(DirectionalLight::new(upwards)).camera(Camera::perspective().look_at(Point3::new(0.0, -3.0, 0.001), Point3::new(0.0, 0.0, 0.0)));
        let [r, g, b, _] = centre(&picture(scene));
        r as u32 + g as u32 + b as u32
    };

    let single_back = plane_centre(false, true, upwards);
    let back_lit = plane_centre(true, true, upwards);
    let back_unlit = plane_centre(true, true, downwards);
    let front_lit = plane_centre(true, false, downwards);
    let single_front_lit = plane_centre(false, false, downwards);

    assert_eq!(single_back, background, "a single-sided plane's back is drawn");
    assert!(back_lit > back_unlit + 200, "the back isn't lit from below: {back_lit} against {back_unlit} lit from above");
    assert!(back_lit.abs_diff(front_lit) <= 6, "the back lit from below: {back_lit}, the front from above: {front_lit}");
    assert_eq!(front_lit, single_front_lit, "being double-sided changes the front");
}

/// Two upright planes with their backs to the camera, lit from the camera's side, over a floor:
/// the single-sided one on the left isn't drawn, but still casts its shadow; the double-sided one
/// on the right shows its back, lit.
#[test]
fn a_double_sided_plane_from_behind() {
    // Turned to face away from the camera, along -Z.
    let upright = |double_sided: bool, x: f32| {
        Object::new(Mesh::plane(1.0, 1.0), PbrMaterial::new().color(ORANGE).roughness(Some(0.6f32)).double_sided(double_sided))
            .rotation(Euler::new(Deg(-90.0), Deg(0.0), Deg(0.0)))
            .position(Vector3::new(x, 0.5, 0.0))
    };
    let floor = Object::new(Mesh::plane(4.0, 3.0), PbrMaterial::new().color(WHITE).roughness(Some(0.9f32)));

    let scene = Scene3d::new((
        upright(false, -0.7),
        upright(true, 0.7),
        floor,
        DirectionalLight::new(Vector3::new(-0.4, -0.8, -1.0)).intensity(4.0),
    ))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 1.6, 3.2), Point3::new(0.0, 0.3, 0.0)));

    let mut harness = Harness::new(framed(scene), Dimension::new(320.0, 220.0));
    harness.snapshot("double_sided");
}
