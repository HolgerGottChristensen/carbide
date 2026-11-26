//! Picking: hover and clicks on nodes, read from the renderer's picking buffer.
//!
//! The scenes are seen through an orthographic camera 2 metres tall in a 200 × 200 view, so a
//! metre is 100 pixels: a unit cube at the origin covers pixels 50 to 150 both ways.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Group, Mesh, Node3d, Node3dExt, Object, PickHit, PickingExt, Scene3d};
use carbide_core::color::WHITE;
use carbide_core::draw::{Dimension, Position};
use carbide_testing::MouseButton;
use carbide_core::math::{InnerSpace, Point3, Vector3};
use carbide_core::state::{LocalState, ReadState, State};
use carbide_core::widget::Widget;
use carbide_testing::Harness;

mod common;

fn cube() -> Object {
    Object::new(Mesh::cube(1.0), PbrMaterial::new().color(WHITE))
}

fn light() -> impl Node3d {
    DirectionalLight::new(Vector3::new(-0.5, -1.0, -0.8)).intensity(4.0)
}

/// `nodes` from in front, orthographic, a metre to 100 pixels.
fn scene<N: carbide_3d::Node3dSequence>(nodes: N) -> Scene3d<N> {
    Scene3d::new(nodes)
        .camera(Camera::orthographic(2.0).look_at(Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 0.0, 0.0)))
}

fn rendered(widget: impl Widget) -> Harness {
    let mut harness = Harness::new(widget, Dimension::new(200.0, 200.0));
    harness.render();
    // A view renders its picking buffer only while the pointer is inside it: move in, off the
    // cube, and render the frame that pointer events then read, as entering a view asks for.
    harness.mouse_move(OUTSIDE);
    harness.render();
    harness
}

const CENTRE: Position = Position::new(100.0, 100.0);
/// Move the pointer to `position` and render a frame: hover is picked once a frame, in the
/// update, not on each move.
fn hover(harness: &mut Harness, position: Position) {
    harness.mouse_move(position);
    harness.render();
}

/// Inside the view, but off everything in these scenes.
const OUTSIDE: Position = Position::new(10.0, 10.0);

/// A cube whose `.hovered(..)` state follows the pointer. Pins down hover: true over the cube,
/// false off it.
#[test]
fn hovered_follows_the_pointer() {
    let hovered = LocalState::new(false);
    let mut harness = rendered(scene((cube().hovered(hovered.clone()), light())));

    hover(&mut harness, CENTRE);
    assert!(*hovered.value(), "over the cube");

    hover(&mut harness, OUTSIDE);
    assert!(!*hovered.value(), "off the cube");
}

/// A click on the cube's front face. Pins down the hit: the cube, at the front face's depth
/// (z = 0.5), with a normal facing the camera, 4.5 metres from it.
#[test]
fn a_click_gives_the_hit() {
    let hit = LocalState::new(None::<PickHit>);
    let cube = cube();
    let cube_id = carbide_3d::CommonNode3d::id(&cube);
    let on_click = {
        let hit = hit.clone();
        move |picked: &PickHit, _: &mut carbide_core::environment::Environment| hit.clone().set_value(Some(picked.clone()))
    };
    let mut harness = rendered(scene((cube.on_click(on_click), light())));

    harness.click(Position::new(120.0, 80.0));
    let hit = hit.value().clone().expect("the click to hit the cube");

    assert_eq!(hit.node, cube_id);
    assert!((hit.position - Vector3::new(0.2, 0.2, 0.5)).magnitude() < 0.02, "{:?}", hit.position);
    assert!((hit.distance - 4.5).abs() < 0.02, "{}", hit.distance);
    let normal = hit.normal.expect("a normal from the neighbouring pixels");
    assert!((normal - Vector3::new(0.0, 0.0, 1.0)).magnitude() < 0.01, "{normal:?}");
}

/// Clicks that don't count: on the background, and pressed on the cube but released off it; and
/// a quick second click, which does. Pins down that a click needs a hit and the press and release
/// on the same node, and that each click of a double click counts.
#[test]
fn only_clicks_on_the_node_count() {
    let clicks = LocalState::new(0);
    let on_click = {
        let clicks = clicks.clone();
        move |_: &PickHit, _: &mut carbide_core::environment::Environment| {
            let count = *clicks.value();
            clicks.clone().set_value(count + 1);
        }
    };
    let mut harness = rendered(scene((cube().on_click(on_click), light())));

    harness.click(OUTSIDE);
    assert_eq!(*clicks.value(), 0, "a click on the background");

    harness.mouse_move(CENTRE);
    harness.mouse_press(MouseButton::Left);
    harness.mouse_move(OUTSIDE);
    harness.mouse_release(MouseButton::Left);
    assert_eq!(*clicks.value(), 0, "pressed on the cube, released off it");

    harness.click(CENTRE);
    assert_eq!(*clicks.value(), 1, "a click on the cube");

    // Straight after, at the same place: a double click, which counts as a second click.
    harness.click(CENTRE);
    assert_eq!(*clicks.value(), 2, "a second, quick click");
}

/// A group with a click handler around a cube with its own. Pins down bubbling: both hear the
/// click, the cube first.
#[test]
fn clicks_bubble_innermost_first() {
    let order = LocalState::new(Vec::<&'static str>::new());
    let record = |name: &'static str| {
        let order = order.clone();
        move |_: &PickHit, _: &mut carbide_core::environment::Environment| {
            let mut names = order.value().clone();
            names.push(name);
            order.clone().set_value(names);
        }
    };

    let mut harness = rendered(scene((Group::new(cube().on_click(record("cube"))).on_click(record("group")), light())));

    harness.click(CENTRE);
    assert_eq!(*order.value(), vec!["cube", "group"]);
}

/// A cube in front of a hovered sphere, not pickable, and then a part of a hovered group made
/// unpickable. Pins down that unpickable meshes still hide what is behind them, and that
/// `.pickable(false)` excludes part of a handler's subtree.
#[test]
fn unpickable_meshes_hide_and_are_excluded() {
    let hovered = LocalState::new(false);
    let sphere = Object::new(Mesh::icosphere(0.8, 3), PbrMaterial::new().color(WHITE))
        .position(Vector3::new(0.0, 0.0, -1.0))
        .hovered(hovered.clone());
    let mut harness = rendered(scene((sphere, cube(), light())));

    hover(&mut harness, CENTRE);
    assert!(!*hovered.value(), "the cube in front hides the sphere");
    // Beside the cube, the sphere shows.
    hover(&mut harness, Position::new(100.0, 170.0));
    assert!(*hovered.value(), "the sphere below the cube");

    let hovered = LocalState::new(false);
    let group = Group::new((
        cube().position(Vector3::new(-0.6, 0.0, 0.0)).pickable(false),
        cube().scale(0.5f32).position(Vector3::new(0.6, 0.0, 0.0)),
    )).hovered(hovered.clone());
    let mut harness = rendered(scene((group, light())));

    hover(&mut harness, Position::new(40.0, 100.0));
    assert!(!*hovered.value(), "the unpickable cube");
    hover(&mut harness, Position::new(160.0, 100.0));
    assert!(*hovered.value(), "the pickable one");
}

/// The pointer 2 pixels beside the cube's right edge (pixel 151; the cube ends at 150), with the
/// default tolerance and with none. Pins down the tolerance: the nearest pickable pixel within 2
/// pixels counts, and with a tolerance of 0 only the pixel under the pointer does.
#[test]
fn the_tolerance_reaches_beside_an_edge() {
    let beside = Position::new(151.5, 100.0);

    let hovered = LocalState::new(false);
    let mut harness = rendered(scene((cube().hovered(hovered.clone()), light())));
    hover(&mut harness, beside);
    assert!(*hovered.value(), "within the default 2 pixels");

    let hovered = LocalState::new(false);
    let mut harness = rendered(scene((cube().hovered(hovered.clone()), light())).pick_tolerance(0.0));
    hover(&mut harness, beside);
    assert!(!*hovered.value(), "with no tolerance");
}

/// A cube moved under a still pointer, and away again, through a position state. Pins down that
/// hover follows the scene, not only the pointer: each update picks again at the pointer against
/// the last frame, so hover changes one frame after the scene does.
#[test]
fn hover_follows_the_scene_under_a_still_pointer() {
    let hovered = LocalState::new(false);
    let position = LocalState::new(Vector3::new(-1.5f32, 0.0, 0.0));
    let mut harness = rendered(scene((cube().position(position.clone()).hovered(hovered.clone()), light())));

    // The pointer stays at the centre; the cube starts off to the left.
    hover(&mut harness, CENTRE);
    assert!(!*hovered.value());

    // The cube moves under the pointer: the next frame renders it, and the one after picks it.
    position.clone().set_value(Vector3::new(0.0, 0.0, 0.0));
    harness.render();
    harness.render();
    assert!(*hovered.value(), "the cube moved under the pointer");

    // And away again.
    position.clone().set_value(Vector3::new(1.5, 0.0, 0.0));
    harness.render();
    harness.render();
    assert!(!*hovered.value(), "the cube moved away");
}

/// The hover and click tests' cube on a display with a scale factor of 2, as a retina screen.
/// Pins down that picks convert the pointer's logical position to the layer's physical pixels.
#[test]
fn picking_on_a_scaled_display() {
    let hovered = LocalState::new(false);
    let mut harness = Harness::new_with_options(
        scene((cube().hovered(hovered.clone()), light())),
        Dimension::new(200.0, 200.0),
        2.0,
        carbide_testing::WgpuMsaa::X1,
    );
    harness.render();
    harness.mouse_move(OUTSIDE);
    harness.render();

    hover(&mut harness, Position::new(120.0, 80.0));
    assert!(*hovered.value(), "over the cube");

    hover(&mut harness, Position::new(170.0, 100.0));
    assert!(!*hovered.value(), "beside the cube");

    hover(&mut harness, Position::new(60.0, 140.0));
    assert!(*hovered.value(), "over the cube's other corner");
}

/// A hovered cube in a view whose `interactive` state is switched off, then on. Pins down that a
/// view that isn't interactive lets go of the pointer, so hover ends, and then ignores it: no
/// hover and no clicks, until it is interactive again.
#[test]
fn a_view_that_isnt_interactive_ignores_the_pointer() {
    let hovered = LocalState::new(false);
    let clicks = LocalState::new(0);
    let interactive = LocalState::new(true);
    let on_click = {
        let clicks = clicks.clone();
        move |_: &PickHit, _: &mut carbide_core::environment::Environment| {
            let count = *clicks.value();
            clicks.clone().set_value(count + 1);
        }
    };
    let mut harness = rendered(scene((cube().hovered(hovered.clone()).on_click(on_click), light())).interactive(interactive.clone()));

    hover(&mut harness, CENTRE);
    assert!(*hovered.value(), "hovered while interactive");

    interactive.clone().set_value(false);
    harness.render();
    assert!(!*hovered.value(), "hover ends when the view stops being interactive");

    hover(&mut harness, Position::new(110.0, 100.0));
    harness.click(CENTRE);
    assert!(!*hovered.value(), "no hover");
    assert_eq!(*clicks.value(), 0, "no clicks");

    interactive.clone().set_value(true);
    harness.render();
    hover(&mut harness, Position::new(105.0, 100.0));
    harness.render();
    assert!(*hovered.value(), "hovered again once interactive");
}

/// Clicks on the background, on the cube, a drag across the background, and a press on the cube
/// released on the background, in a view with a missed-click handler. Pins down `on_missed_click`:
/// only a click pressed and released on nothing pickable counts, and a click on the cube goes to
/// the cube.
#[test]
fn a_click_on_nothing_is_a_missed_click() {
    let missed = LocalState::new(0);
    let clicks = LocalState::new(0);
    let count = |state: &LocalState<i32>| {
        let state = state.clone();
        move || {
            let count = *state.value();
            state.clone().set_value(count + 1);
        }
    };
    let (on_missed, on_cube) = (count(&missed), count(&clicks));
    let scene = scene((cube().on_click(move |_: &PickHit, _: &mut carbide_core::environment::Environment| on_cube()), light()))
        .on_missed_click(move |_: &mut carbide_core::environment::Environment| on_missed());
    let mut harness = rendered(scene);

    harness.click(OUTSIDE);
    assert_eq!((*missed.value(), *clicks.value()), (1, 0), "a click on the background");

    harness.click(CENTRE);
    assert_eq!((*missed.value(), *clicks.value()), (1, 1), "a click on the cube");

    harness.drag(Position::new(20.0, 20.0), Position::new(40.0, 20.0), 3);
    assert_eq!(*missed.value(), 1, "a drag isn't a click");

    hover(&mut harness, CENTRE);
    harness.mouse_press(MouseButton::Left);
    harness.mouse_move(OUTSIDE);
    harness.mouse_release(MouseButton::Left);
    assert_eq!(*missed.value(), 1, "pressed on the cube, released on the background");
}
