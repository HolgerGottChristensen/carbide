//! Image-based lighting with real environments: Poly Haven's `studio_small_09` and
//! `rural_asphalt_road` (`ressources/hdri`, CC0, 1k), loaded from their `.hdr` files. Snapshots of
//! spheres across roughness and metalness, the damaged helmet, the rotation, the intensity and the
//! background, with checks of what each should show.

use std::path::PathBuf;

use carbide_3d::camera::Camera;
use carbide_3d::gltf::{load_gltf, GltfModel};
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::render::Background;
use carbide_3d::{Mesh, Node3dExt, Node3dSequence, Object, Scene3d};
use carbide_core::color::{rgba_bytes, Color, WHITE};
use carbide_core::draw::{Dimension, ImageId};
use carbide_core::math::{Matrix4, Point3, Vector3};
use carbide_core::widget::{HStack, WidgetExt};
use carbide_testing::Harness;

mod common;

const STUDIO: &str = "studio_small_09";
const ROAD: &str = "rural_asphalt_road";

/// The HDRI `name` in `ressources/hdri`.
fn hdri(name: &str) -> ImageId {
    ImageId::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ressources/hdri").join(name).join(format!("{name}_1k.hdr")))
}

/// The mean brightness, 0 to 765, of the pixels within `radius` of (`x`, `y`).
fn brightness(harness: &mut Harness, x: u32, y: u32, radius: u32) -> u32 {
    let picture = harness.pixels();
    let mut sum = 0;
    let mut count = 0;
    for py in y - radius..=y + radius {
        for px in x - radius..=x + radius {
            let [r, g, b, _] = picture.get_pixel(px, py).0;
            sum += r as u32 + g as u32 + b as u32;
            count += 1;
        }
    }
    sum / count
}

/// Two rows of five spheres, roughness 0 to 1 from left to right: metal, silver-white, above;
/// red plastic below. Seen from the front in a 500 × 220 view, each sphere's middle at
/// (50 + 100 i, 60) above and (50 + 100 i, 160) below.
fn grid() -> impl Node3dSequence {
    let sphere = |row: f32, column: usize, material: PbrMaterial| {
        Object::new(Mesh::icosphere(0.42, 5), material).position(Vector3::new(column as f32 - 2.0, row, 0.0))
    };
    let roughness = |column: usize| Some(column as f32 / 4.0);
    let metal = |column| PbrMaterial::new().color(rgba_bytes(245, 245, 245, 1.0)).metallic(1.0).roughness(roughness(column));
    let plastic = |column| PbrMaterial::new().color(rgba_bytes(200, 30, 30, 1.0)).roughness(roughness(column));

    (0..5).flat_map(|column| [sphere(0.5, column, metal(column)), sphere(-0.5, column, plastic(column))]).collect::<Vec<_>>()
}

fn grid_view(environment: &str) -> Harness {
    let scene = Scene3d::new(grid())
        .environment(Some(hdri(environment)))
        .background(Background::Environment { blur: 0.5 })
        .camera(Camera::orthographic(2.2).look_at(Point3::new(0.0, 0.0, 6.0), Point3::new(0.0, 0.0, 0.0)));
    let mut harness = Harness::new(scene, Dimension::new(500.0, 220.0));
    harness.render();
    harness
}

/// Lit by the environment alone, the metal spheres show it, sharp when smooth and blurred when
/// rough: inside a smooth metal sphere, neighbouring pixels differ far more than inside a rough
/// one (a sun's highlight makes a rough sphere's brightness vary too, but smoothly). The plastic
/// spheres are red all over, with a reflection that fades as they roughen.
fn check_grid(harness: &mut Harness) {
    let picture = harness.pixels();
    // Summed differences between neighbouring pixels in the 40 × 40 square around (x, y).
    let detail = |x: u32, y: u32| -> u32 {
        (y - 20..y + 20).flat_map(|py| (x - 20..x + 20).map(move |px| (px, py)))
            .map(|(px, py)| {
                let a = picture.get_pixel(px, py).0;
                let b = picture.get_pixel(px + 1, py).0;
                (0..3).map(|c| a[c].abs_diff(b[c]) as u32).sum::<u32>()
            })
            .sum()
    };
    let (smooth, rough) = (detail(50, 60), detail(450, 60));
    // Twice as much: the road's sun, filtered with too few samples, once left blotches that
    // brought the rough sphere's detail to three quarters of the smooth one's.
    assert!(smooth > rough * 2, "the smooth metal sphere has no more detail than the rough one: {smooth} against {rough}");

    for column in 0..5 {
        let [r, g, b, _] = picture.get_pixel(50 + 100 * column, 175).0;
        assert!(r > g + 30 && r > b + 30, "plastic sphere {column} isn't red: {:?}", [r, g, b]);
    }
}

#[test]
fn spheres_in_the_studio() {
    let mut harness = grid_view(STUDIO);
    check_grid(&mut harness);
    harness.snapshot("grid");
}

#[test]
fn spheres_on_the_road() {
    let mut harness = grid_view(ROAD);
    check_grid(&mut harness);
    harness.snapshot("grid");
}

/// The damaged helmet, Khronos's sample model, lit only by an environment, from the front a little
/// to the right, with the environment blurred behind it: the visor reflects it, and the scuffed
/// metal shows it rough. Centred on its bounds; its own size, about 2 metres across, kept.
fn helmet(environment: &str) -> Harness {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ressources/gltf/DamagedHelmet/DamagedHelmet.glb");
    let scene = load_gltf(path).expect("the helmet loads");
    let (min, max) = scene.bounds().expect("the helmet has vertices");
    let centre = Vector3::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0, (min.z + max.z) / 2.0);

    let view = Scene3d::new(GltfModel::new(scene).transform(Matrix4::from_translation(-centre)))
        .environment(Some(hdri(environment)))
        .background(Background::Environment { blur: 0.3 })
        .camera(Camera::perspective().look_at(Point3::new(1.2, 0.5, 3.0), Point3::new(0.0, 0.0, 0.0)));
    let mut harness = Harness::new(view, Dimension::new(300.0, 300.0));
    harness.render();
    harness
}

#[test]
fn the_helmet_in_the_studio() {
    helmet(STUDIO).snapshot("helmet");
}

#[test]
fn the_helmet_on_the_road() {
    helmet(ROAD).snapshot("helmet");
}

/// A chrome sphere on the road at three rotations, 0°, 120° and 240°, with the environment sharp
/// behind it: the reflection and the background turn together, so each view differs from the
/// next.
#[test]
fn the_road_turned() {
    let view = |degrees: f32| Scene3d::new(Object::new(Mesh::icosphere(1.0, 5), PbrMaterial::new().color(WHITE).metallic(1.0).roughness(Some(0.05f32))))
        .environment(Some(hdri(ROAD)))
        .environment_rotation(degrees)
        .background(Background::Environment { blur: 0.0 })
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 3.5), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((view(0.0), view(120.0), view(240.0))).spacing(0.0), Dimension::new(600.0, 200.0));
    harness.render();

    let picture = harness.pixels();
    let differing = |offset: u32| (0..200).flat_map(|y| (0..200).map(move |x| (x, y)))
        .filter(|(x, y)| picture.get_pixel(*x, *y).0 != picture.get_pixel(*x + offset, *y).0)
        .count();
    assert!(differing(200) > 10_000 && differing(400) > 10_000, "turning hardly changes the view");
    harness.snapshot("turned");
}

/// A gold sphere and a white plastic one in the studio at intensities 0.5, 1 and 2: each step
/// brighter than the last, the background too.
#[test]
fn the_studio_at_three_intensities() {
    let gold: Color = rgba_bytes(255, 195, 86, 1.0);
    let view = |intensity: f32| Scene3d::new((
        Object::new(Mesh::icosphere(0.45, 5), PbrMaterial::new().color(gold).metallic(1.0).roughness(Some(0.3f32))).position(Vector3::new(-0.55, 0.0, 0.0)),
        Object::new(Mesh::icosphere(0.45, 5), PbrMaterial::new().color(WHITE).roughness(Some(0.6f32))).position(Vector3::new(0.55, 0.0, 0.0)),
    ))
        .environment(Some(hdri(STUDIO)))
        .environment_intensity(intensity)
        .background(Background::Environment { blur: 0.6 })
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 2.8), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((view(0.5), view(1.0), view(2.0))).spacing(0.0), Dimension::new(600.0, 200.0));
    harness.render();

    // The white sphere's middle, and the background's top left corner, in each view.
    let white: Vec<u32> = (0..3).map(|view| brightness(&mut harness, 137 + 200 * view, 100, 3)).collect();
    let background: Vec<u32> = (0..3).map(|view| brightness(&mut harness, 10 + 200 * view, 10, 3)).collect();
    assert!(white[0] < white[1] && white[1] < white[2], "the white sphere doesn't brighten: {white:?}");
    assert!(background[0] < background[1] && background[1] < background[2], "the background doesn't brighten: {background:?}");
    harness.snapshot("intensities");
}

/// The road as an empty scene's background, looking along the road at the horizon, at blurs of 0,
/// 0.3 and 1: the sharp one has detail, which fades as the blur rises.
#[test]
fn the_road_as_a_background() {
    let view = |blur: f32| Scene3d::new(())
        .environment(Some(hdri(ROAD)))
        .background(Background::Environment { blur })
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, -1.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((view(0.0), view(0.3), view(1.0))).spacing(0.0), Dimension::new(600.0, 200.0));
    harness.render();

    // Detail: how much neighbouring pixels differ, summed over the view.
    let picture = harness.pixels();
    let detail = |offset: u32| -> u32 {
        (0..199).flat_map(|y| (0..199).map(move |x| (x, y)))
            .map(|(x, y)| {
                let a = picture.get_pixel(x + offset, y).0;
                let b = picture.get_pixel(x + offset + 1, y + 1).0;
                (0..3).map(|c| a[c].abs_diff(b[c]) as u32).sum::<u32>()
            })
            .sum()
    };
    let (sharp, soft, blurred) = (detail(0), detail(200), detail(400));
    assert!(sharp > soft && soft > blurred, "the blur doesn't remove detail: {sharp}, {soft}, {blurred}");
    harness.snapshot("blurs");
}
