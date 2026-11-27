//! Blended transparency: blended meshes drawn after the opaque ones, farthest first, without
//! writing depth; premultiplied output; double-sided blended meshes drawn back faces first; and
//! shadows alpha-tested for cutout and blended materials.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::material::transparency::Transparency;
use carbide_3d::{Mesh, Node3d, Node3dExt, Node3dSequence, Object, Scene3d};
use carbide_core::color::{rgba, rgba_bytes, ORANGE, TRANSPARENT, WHITE};
use carbide_core::draw::{Color, Dimension, ImageContext, ImageFormat, ImageId, Texture, TextureFormat};
use carbide_core::math::{Deg, Euler, Point3, Vector3};
use carbide_core::state::{AnyReadState, IntoReadState, LocalState, Map1, State};
use carbide_3d::material::Material;
use carbide_core::widget::{HStack, Rectangle, WidgetExt, ZStack};
use carbide_testing::Harness;
use carbide_wgpu_core::WgpuImageContext;

mod common;

/// A blended, unlit square of `size` metres, facing the camera (+Z), at `position`, in `color`
/// with its alpha.
fn pane(color: Color, size: f32, position: Vector3<f32>) -> impl Node3d {
    Object::new(Mesh::plane(size, size), PbrMaterial::new().color(color).unlit().transparency(Transparency::Blend))
        .rotation(Euler::new(Deg(90.0), Deg(0.0), Deg(0.0)))
        .position(position)
}

/// `nodes` seen straight on, orthographic, 2 metres tall in a 200 × 200 view: a metre is 100
/// pixels, and the origin is the middle.
fn straight_on<N: Node3dSequence>(nodes: N) -> Scene3d<N> {
    Scene3d::new(nodes).camera(Camera::orthographic(2.0).look_at(Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 0.0, 0.0)))
}

fn rendered(scene: impl carbide_core::widget::Widget) -> Harness {
    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    harness
}

fn pixel(picture: &image::RgbaImage, x: u32, y: u32) -> [u8; 4] {
    picture.get_pixel(x, y).0
}

/// Three half-transparent panes, red farthest and blue nearest, overlapping. Sorted back to front,
/// the picture is the same whichever order the scene lists them in, and every pane shows through
/// the nearer ones: none writes depth, so none hides another.
#[test]
fn blended_meshes_are_sorted_back_to_front() {
    let red = || pane(rgba(1.0, 0.0, 0.0, 0.5), 1.0, Vector3::new(-0.3, 0.3, -1.0));
    let green = || pane(rgba(0.0, 1.0, 0.0, 0.5), 1.0, Vector3::new(0.0, 0.0, 0.0));
    let blue = || pane(rgba(0.0, 0.0, 1.0, 0.5), 1.0, Vector3::new(0.3, -0.3, 1.0));

    let mut back_first = rendered(straight_on((red(), green(), blue())));
    let mut front_first = rendered(straight_on((blue(), green(), red())));
    let picture = back_first.pixels();
    assert!(picture.as_raw() == front_first.pixels().as_raw(), "the order in the scene changes the picture");

    // The middle is covered by all three: each shows.
    let [r, g, b, _] = pixel(&picture, 100, 100);
    assert!(r > 30 && g > 30 && b > 60, "the middle is {:?}", [r, g, b]);
    back_first.snapshot("overlapping");
}

/// An opaque white square and a half-transparent red one across it. With the red one in front,
/// the white shows through it, pink; with the red one behind, the white hides it, since blended
/// meshes are tested against the opaque ones' depth.
#[test]
fn opaque_meshes_hide_blended_ones_behind_them() {
    let white = || Object::new(Mesh::plane(1.0, 1.0), PbrMaterial::new().color(WHITE).unlit())
        .rotation(Euler::new(Deg(90.0), Deg(0.0), Deg(0.0)))
        .position(Vector3::new(-0.3, 0.0, 0.0));
    let red = |z: f32| pane(rgba(1.0, 0.0, 0.0, 0.5), 1.0, Vector3::new(0.3, 0.0, z));

    let mut in_front = rendered(straight_on((white(), red(1.0))));
    let mut behind = rendered(straight_on((white(), red(-1.0))));

    // Where they overlap.
    let [r, g, b, _] = pixel(&in_front.pixels(), 100, 100);
    assert!(r > 240 && g > 100 && g < 230 && (g as i32 - b as i32).abs() <= 2, "in front: {:?}, not pink", [r, g, b]);
    assert_eq!(&pixel(&behind.pixels(), 100, 100)[..3], &[255, 255, 255], "behind: the red one shows through the white");
    in_front.snapshot("in_front");
}

/// A scene with a transparent background over a white 2D rectangle, with a half-transparent red
/// pane: the layer is premultiplied, so the pane composes over the white as pink, and the rest of
/// the scene leaves the white as it is.
#[test]
fn blended_meshes_compose_over_the_ui() {
    let scene = straight_on(pane(rgba(1.0, 0.0, 0.0, 0.5), 1.0, Vector3::new(0.0, 0.0, 0.0))).background(TRANSPARENT);
    let mut harness = rendered(ZStack::new((Rectangle::new().fill(WHITE), scene)));

    let picture = harness.pixels();
    let [r, g, b, _] = pixel(&picture, 100, 100);
    assert!(r > 240 && g > 100 && g < 230 && (g as i32 - b as i32).abs() <= 2, "over the white: {:?}, not pink", [r, g, b]);
    assert_eq!(&pixel(&picture, 10, 10)[..3], &[255, 255, 255], "the empty scene covers the white");
    harness.snapshot("over_white");
}

/// A lit, orange glass sphere, single-sided on the left and double-sided on the right: the
/// double-sided one shows its far side through its near side, drawn first.
#[test]
fn a_double_sided_blended_sphere_shows_its_far_side() {
    let sphere = |double_sided: bool| {
        let glass = PbrMaterial::new()
            .color(rgba_bytes(255, 140, 0, 0.35))
            .roughness(Some(0.3f32))
            .transparency(Transparency::Blend)
            .double_sided(double_sided);
        Scene3d::new((Object::new(Mesh::icosphere(1.0, 4), glass), DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0)).intensity(4.0)))
            .ambient(WHITE, 0.3)
            .camera(Camera::perspective().look_at(Point3::new(0.0, 0.5, 3.2), Point3::new(0.0, 0.0, 0.0)))
            .frame(200.0, 200.0)
    };

    let mut harness = Harness::new(HStack::new((sphere(false), sphere(true))).spacing(0.0), Dimension::new(400.0, 200.0));
    harness.render();
    let picture = harness.pixels();
    let changed = (60..140).flat_map(|y| (60..140).map(move |x| (x, y)))
        .filter(|(x, y)| pixel(&picture, *x, *y) != pixel(&picture, *x + 200, *y))
        .count();
    assert!(changed > 3_000, "the far side changes only {changed} pixels of the middle");
    harness.snapshot("single_and_double_sided");
}

/// A 16 × 16 image, opaque white with a transparent hole in its middle half, stored under a new
/// id.
fn store_hole(harness: &mut Harness) -> ImageId {
    let mut data = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            let hole = (4..12).contains(&x) && (4..12).contains(&y);
            data.extend_from_slice(if hole { &[0, 0, 0, 0] } else { &[255, 255, 255, 255] });
        }
    }
    let id = ImageId::temp(ImageFormat::Png);
    let texture = Texture { width: 16, height: 16, bytes_per_row: 64, format: TextureFormat::RGBA8, data: &data };
    WgpuImageContext.update_texture(&id, texture, harness.environment_mut());
    id
}

/// Three squares lying flat half a metre above a white floor, lit from the left, so their shadows
/// fall half a metre to their right, seen from straight above. Left: a cutout square with a hole
/// in its middle half, whose shadow has the hole too. Middle: blended at an alpha of 0.3, under
/// 0.5, which casts no shadow. Right: blended at 0.8, which casts one.
#[test]
fn cutout_and_blended_shadows_are_alpha_tested() {
    let hole = LocalState::new(None::<ImageId>);
    let square = |material: Box<dyn AnyReadState<T=Material>>, x: f32| Object::new(Mesh::plane(0.5, 0.5), material).position(Vector3::new(x, 0.5, 0.0));
    let blended = |alpha: f32| IntoReadState::<Material>::into_dyn_read_state(PbrMaterial::new().color(rgba(1.0, 0.55, 0.0, alpha)).transparency(Transparency::Blend));
    let cutout = Map1::read_map(hole.clone(), |id: &Option<ImageId>| {
        let albedo = match id {
            Some(id) => carbide_3d::material::albedo_component::AlbedoComponent::TextureValue { texture: id.into(), value: ORANGE },
            None => carbide_3d::material::albedo_component::AlbedoComponent::Value(ORANGE),
        };
        PbrMaterial::new().color(albedo).transparency(Transparency::Cutout { cutout: 0.5 })
    });
    let cutout = IntoReadState::<Material>::into_dyn_read_state(cutout);

    let scene = Scene3d::new((
        Object::new(Mesh::plane(4.0, 4.0), PbrMaterial::new().color(WHITE).roughness(Some(0.9f32))),
        square(cutout, -1.2),
        square(blended(0.3), 0.0),
        square(blended(0.8), 1.2),
        DirectionalLight::new(Vector3::new(1.0, -1.0, 0.0)).intensity(3.0),
    ))
        .ambient(WHITE, 0.15)
        .camera(Camera::orthographic(4.4).look_at(Point3::new(0.0, 5.0, 0.001), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    let id = store_hole(&mut harness);
    *hole.clone().value_mut() = Some(id);
    harness.render();

    // 200 pixels for 4.4 metres; the origin in the middle. Brightness of the floor at x metres,
    // and of a lit spot 0.8 metres off the shadows' row.
    let picture = harness.pixels();
    let at = |x: f32, z: f32| {
        let [r, g, b, _] = pixel(&picture, (100.0 + x * 200.0 / 4.4).round() as u32, (100.0 + z * 200.0 / 4.4).round() as u32);
        r as u32 + g as u32 + b as u32
    };
    let lit = at(-0.7, 0.8);
    let shadowed = |brightness: u32| brightness + 150 < lit;

    assert!(shadowed(at(-0.9, 0.0)), "no shadow from the cutout square's edge: {} against {lit}", at(-0.9, 0.0));
    assert!(!shadowed(at(-0.7, 0.0)), "a shadow under the cutout square's hole: {} against {lit}", at(-0.7, 0.0));
    assert!(!shadowed(at(0.5, 0.0)), "the square at 0.3 casts a shadow: {} against {lit}", at(0.5, 0.0));
    assert!(shadowed(at(1.7, 0.0)), "the square at 0.8 casts no shadow: {} against {lit}", at(1.7, 0.0));
    harness.snapshot("shadows");
}

