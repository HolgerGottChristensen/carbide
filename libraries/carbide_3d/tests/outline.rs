//! Outlines around outlined nodes, inherited through the environment.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Group, Mesh, Node3d, Node3dExt, Object, OutlineExt, Scene3d};
use carbide_core::color::{rgba_bytes, BLUE, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::math::{Point3, Vector3};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{Widget, WidgetExt};
use carbide_testing::Harness;

mod common;

fn framed(scene: impl Widget) -> impl Widget {
    scene.border().color(BLUE).padding(20.0)
}

/// A pure green, unlike anything else in these scenes, so outline pixels can be counted.
fn green() -> carbide_core::color::Color {
    rgba_bytes(0, 255, 0, 1.0)
}

fn light() -> impl Node3d {
    DirectionalLight::new(Vector3::new(-0.5, -1.0, -0.8)).intensity(4.0)
}

fn cube() -> Object {
    Object::new(Mesh::cube(1.0), PbrMaterial::new().color(WHITE))
}

fn camera() -> Camera {
    Camera::perspective().look_at(Point3::new(1.5, 1.5, 3.0), Point3::new(0.0, 0.0, 0.0))
}

/// The rendered picture's bytes, 200 by 200.
fn picture(scene: impl Widget) -> Vec<u8> {
    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    harness.pixels().as_raw().to_vec()
}

/// How many pixels are about the green of the outlines.
fn green_pixels(picture: &[u8]) -> usize {
    picture.chunks_exact(4).filter(|p| p[1] > 200 && p[0] < 60 && p[2] < 60).count()
}

fn difference(a: &[u8], b: &[u8]) -> u8 {
    a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0)
}

/// A white cube outlined in green, 3 pixels thick. Pins down the outline: a band of the colour
/// around the cube's silhouette, not over the cube, with a soft outer edge.
#[test]
fn an_outlined_cube() {
    let scene = Scene3d::new((cube().outlined(true).outline_color(green()).outline_thickness(3.0), light())).camera(camera());

    let mut harness = Harness::new(framed(scene), Dimension::new(300.0, 300.0));
    harness.snapshot("cube");
}

/// The cube with nothing set, outlined off, and outlined on. Pins down that nothing is outlined
/// by default, and that the outline adds pixels of its colour.
#[test]
fn nothing_is_outlined_by_default() {
    let scene = |outlined: Option<bool>| {
        let cube = cube().outline_color(green());
        match outlined {
            None => Scene3d::new((cube.boxed(), light().boxed())).camera(camera()),
            Some(outlined) => Scene3d::new((cube.outlined(outlined).boxed(), light().boxed())).camera(camera()),
        }
    };

    let unset = picture(scene(None));
    let off = picture(scene(Some(false)));
    let on = picture(scene(Some(true)));

    assert_eq!(difference(&unset, &off), 0);
    assert_eq!(green_pixels(&off), 0);
    assert!(green_pixels(&on) > 100, "only {} outline pixels", green_pixels(&on));
}

/// The outline switched through a state. Pins down that it is reactive.
#[test]
fn the_outline_follows_a_state() {
    let outlined = LocalState::new(false);
    let scene = Scene3d::new((cube().outlined(outlined.clone()).outline_color(green()), light())).camera(camera());

    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    let before = green_pixels(harness.pixels().as_raw());

    outlined.clone().set_value(true);
    harness.render();
    let after = green_pixels(harness.pixels().as_raw());

    assert_eq!(before, 0);
    assert!(after > 100);
}

/// The colour and thickness set on the scene widget, and on the node. Pins down that the style is
/// inherited from widgets too, and that a thicker outline covers more pixels.
#[test]
fn the_style_is_inherited() {
    let on_widget = picture(Scene3d::new((cube().outlined(true), light())).camera(camera()).outline_color(green()).outline_thickness(4.0));
    let on_node = picture(Scene3d::new((cube().outlined(true).outline_color(green()).outline_thickness(4.0), light())).camera(camera()));
    let thin = picture(Scene3d::new((cube().outlined(true).outline_color(green()).outline_thickness(1.0), light())).camera(camera()));

    assert_eq!(difference(&on_widget, &on_node), 0);
    assert!(green_pixels(&on_node) > green_pixels(&thin) * 2, "4 px: {}, 1 px: {}", green_pixels(&on_node), green_pixels(&thin));
}

/// Two touching cubes in an outlined group, and an outlined sphere behind a cube that isn't.
/// Pins down that touching objects get one outline around them both, and that outlines are drawn
/// on top: the sphere's outline shows across the cube in front of it.
#[test]
fn one_outline_around_a_group_drawn_on_top() {
    let pair = Group::new((
        cube().position(Vector3::new(-0.5, 0.0, 0.0)),
        cube().position(Vector3::new(0.5, 0.0, 0.0)),
    ))
        .outlined(true)
        .position(Vector3::new(-0.4, 0.0, 0.0));

    let hidden = Object::new(Mesh::icosphere(0.6, 3), PbrMaterial::new().color(WHITE))
        .position(Vector3::new(1.4, 0.0, -1.5))
        .outlined(true);
    let blocker = cube().scale(0.9f32).position(Vector3::new(1.4, 0.0, 0.0));

    let scene = Scene3d::new((pair, hidden, blocker, light()))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 1.0, 4.5), Point3::new(0.2, 0.0, 0.0)))
        .outline_color(green())
        .outline_thickness(3.0);

    let mut harness = Harness::new(framed(scene), Dimension::new(400.0, 260.0));
    harness.snapshot("group_and_hidden");
}
