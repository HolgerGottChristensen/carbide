//! Image-based lighting: a scene lit by an environment image, with `Scene3d::environment`,
//! `environment_intensity` and `environment_rotation`. The environments are panoramas made here,
//! as HDR (16-bit float) images.

use carbide_3d::camera::Camera;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::render::{Background, Tonemapping};
use carbide_3d::{Mesh, Object, Scene3d};
use carbide_core::color::WHITE;
use carbide_core::draw::{Dimension, ImageContext, ImageFormat, ImageId, Texture, TextureFormat};
use carbide_core::math::Point3;
use carbide_core::widget::{HStack, Widget, WidgetExt};
use carbide_testing::Harness;
use carbide_wgpu_core::WgpuImageContext;
use half::f16;

mod common;

/// Store a `width` × `height` float panorama under `id`, its colour at (u, v) given by `color`,
/// from 0 to 1 across and down.
fn store_panorama(harness: &mut Harness, id: &ImageId, width: u32, height: u32, color: impl Fn(f32, f32) -> [f32; 3]) {
    let mut data = Vec::with_capacity((width * height * 8) as usize);
    for y in 0..height {
        for x in 0..width {
            let [r, g, b] = color((x as f32 + 0.5) / width as f32, (y as f32 + 0.5) / height as f32);
            for value in [r, g, b, 1.0] {
                data.extend_from_slice(&f16::from_f32(value).to_le_bytes());
            }
        }
    }
    let texture = Texture { width, height, bytes_per_row: width * 8, format: TextureFormat::RGBA16F, data: &data };
    WgpuImageContext.update_texture(id, texture, harness.environment_mut());
}

/// A blue sky over a brown ground, with a red square where +Z is: a quarter of the way right of
/// the middle, on the horizon.
fn sky(u: f32, v: f32) -> [f32; 3] {
    if (u - 0.75).abs() < 0.04 && (v - 0.5).abs() < 0.08 {
        [1.0, 0.0, 0.0]
    } else if v < 0.5 {
        [0.3, 0.5, 1.0]
    } else {
        [0.4, 0.25, 0.1]
    }
}

/// A sphere of radius 1 at the origin, seen straight on from +Z, orthographic, 2.5 metres tall in
/// a 200 × 200 view: the sphere's middle is the view's, and its radius is 80 pixels. Without tone
/// mapping, so colours show as they are, clamped.
fn sphere_scene(material: PbrMaterial) -> Scene3d<impl carbide_3d::Node3dSequence> {
    Scene3d::new(Object::new(Mesh::icosphere(1.0, 5), material))
        .tonemapping(Tonemapping::None)
        .camera(Camera::orthographic(2.5).look_at(Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 0.0, 0.0)))
}

fn mirror() -> PbrMaterial {
    PbrMaterial::new().color(WHITE).metallic(1.0).roughness(Some(0.0f32))
}

fn matte() -> PbrMaterial {
    PbrMaterial::new().color(WHITE).metallic(0.0).roughness(Some(1.0f32))
}

/// Render `widget` in a `width` × 200 window, with the panorama `color` stored under `id` first.
fn rendered(widget: impl Widget, width: f64, id: &ImageId, color: impl Fn(f32, f32) -> [f32; 3]) -> Harness {
    let mut harness = Harness::new(widget, Dimension::new(width, 200.0));
    store_panorama(&mut harness, id, 256, 128, color);
    harness.render();
    harness
}

fn pixel(harness: &mut Harness, x: u32, y: u32) -> [u8; 3] {
    let [r, g, b, _] = harness.pixels().get_pixel(x, y).0;
    [r, g, b]
}

/// A mirror sphere shows the environment: the sky above its middle, the ground below, and the red
/// square, which is behind the camera, in its middle.
#[test]
fn a_mirror_sphere_reflects_the_environment() {
    let id = ImageId::temp(ImageFormat::Png);
    let mut harness = rendered(sphere_scene(mirror()).environment(Some(id.clone())), 200.0, &id, sky);

    let [r, g, b] = pixel(&mut harness, 100, 50);
    assert!(b > r + 40 && b > 150, "above the middle: {:?}, not the sky", [r, g, b]);
    let [r, g, b] = pixel(&mut harness, 100, 150);
    assert!(r > b + 30, "below the middle: {:?}, not the ground", [r, g, b]);
    let [r, g, b] = pixel(&mut harness, 100, 100);
    assert!(r > 200 && g < 80 && b < 80, "the middle: {:?}, not the red square", [r, g, b]);
    harness.snapshot("mirror");
}

/// A white matte sphere in an environment of 0.5 everywhere reflects about 0.5 everywhere, as a
/// white surface under even light: the diffuse light from the harmonics, and a little specular.
/// Without tone mapping, 0.5 is sRGB 188.
#[test]
fn a_matte_sphere_in_an_even_environment_is_evenly_lit() {
    let id = ImageId::temp(ImageFormat::Png);
    let scene = sphere_scene(matte()).environment(Some(id.clone()));
    let mut harness = rendered(scene, 200.0, &id, |_, _| [0.5; 3]);

    for (x, y) in [(100, 100), (60, 100), (100, 150), (150, 70)] {
        let color = pixel(&mut harness, x, y);
        assert!(color.iter().all(|channel| (182..=198).contains(channel)), "at {x}, {y}: {color:?}");
    }
}

/// The environment's intensity scales its light: none at 0, leaving a sphere with no lights
/// black; more at 2 than at 1.
#[test]
fn the_intensity_scales_the_environment_light() {
    let id = ImageId::temp(ImageFormat::Png);
    let scene = |intensity: f32| sphere_scene(matte())
        .environment(Some(id.clone()))
        .environment_intensity(intensity)
        .frame(200.0, 200.0);
    let mut harness = rendered(HStack::new((scene(0.0), scene(1.0), scene(2.0))).spacing(0.0), 600.0, &id, |_, _| [0.2; 3]);

    let none = pixel(&mut harness, 100, 100);
    let one = pixel(&mut harness, 300, 100);
    let two = pixel(&mut harness, 500, 100);
    assert_eq!(none, [0, 0, 0], "lit at an intensity of 0");
    assert!(two[0] > one[0] + 20, "2 isn't brighter than 1: {two:?} and {one:?}");
}

/// Turning the environment 90° counter-clockwise, seen from above, takes the red square from +Z,
/// behind the camera, to +X: the mirror sphere shows it on its right, where the surface faces
/// halfway between the camera and +X, instead of in its middle. A whole turn changes nothing.
#[test]
fn the_rotation_turns_the_environment() {
    let id = ImageId::temp(ImageFormat::Png);
    let scene = |degrees: f32| sphere_scene(mirror()).environment(Some(id.clone())).environment_rotation(degrees).frame(200.0, 200.0);
    let mut harness = rendered(HStack::new((scene(0.0), scene(90.0), scene(360.0))).spacing(0.0), 600.0, &id, sky);

    let red = |[r, g, b]: [u8; 3]| r > 200 && g < 80 && b < 80;
    assert!(red(pixel(&mut harness, 100, 100)), "unturned, the square isn't in the middle");
    // 80 pixels of radius times sin 45°.
    assert!(!red(pixel(&mut harness, 300, 100)), "turned, the square is still in the middle");
    assert!(red(pixel(&mut harness, 357, 100)), "turned, the square isn't on the right: {:?}", pixel(&mut harness, 357, 100));

    let picture = harness.pixels();
    let differing = (0..200).flat_map(|y| (0..200).map(move |x| (x, y)))
        .filter(|(x, y)| picture.get_pixel(*x, *y).0.abs_diff_any(picture.get_pixel(*x + 400, *y).0, 2))
        .count();
    assert!(differing < 20, "a whole turn changes {differing} pixels");
    harness.snapshot("turned");
}

/// The environment is the scene's: a second view of the scene is lit by it too, alike.
#[test]
fn every_view_of_a_scene_shares_its_environment() {
    let id = ImageId::temp(ImageFormat::Png);
    let scene = sphere_scene(mirror()).environment(Some(id.clone())).frame(200.0, 200.0);
    let mut harness = rendered(HStack::new((scene.clone(), scene)).spacing(0.0), 400.0, &id, sky);

    for (x, y) in [(100, 50), (100, 100), (100, 150), (60, 120)] {
        assert_eq!(pixel(&mut harness, x, y), pixel(&mut harness, x + 200, y), "the views differ at {x}, {y}");
    }
}

/// Until its image is loaded, an environment doesn't light the scene: the hemisphere does, as
/// without one.
#[test]
fn an_environment_not_loaded_leaves_the_hemisphere() {
    let missing = ImageId::temp(ImageFormat::Png);
    let unused = ImageId::temp(ImageFormat::Png);
    let mut with = rendered(sphere_scene(matte()).environment(Some(missing)), 200.0, &unused, sky);
    let mut without = rendered(sphere_scene(matte()), 200.0, &unused, sky);
    assert!(with.pixels().as_raw() == without.pixels().as_raw(), "an environment not loaded changes the picture");
}

/// An empty scene with the environment as its background, through a perspective camera at the
/// origin looking along +Z: the red square in the middle, the sky above the horizon, the ground
/// below it.
#[test]
fn the_background_shows_the_environment() {
    let id = ImageId::temp(ImageFormat::Png);
    let scene = Scene3d::new(())
        .environment(Some(id.clone()))
        .background(Background::Environment { blur: 0.0 })
        .tonemapping(Tonemapping::None)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 1.0)));
    let mut harness = rendered(scene, 200.0, &id, sky);

    let [r, g, b] = pixel(&mut harness, 100, 100);
    assert!(r > 200 && g < 80 && b < 80, "the middle: {:?}, not the red square", [r, g, b]);
    let [r, _, b] = pixel(&mut harness, 30, 30);
    assert!(b > r + 40, "above the horizon: {:?}, not the sky", [r, b]);
    let [r, _, b] = pixel(&mut harness, 30, 170);
    assert!(r > b + 30, "below the horizon: {:?}, not the ground", [r, b]);
    harness.snapshot("empty_scene");
}

/// A gold mirror sphere in front of the environment, as the background: the sphere hides it where
/// it is drawn, and reflects what is behind the camera; around it the background shows the sky and
/// the ground beyond the sphere. A white mirror would match the background at its rim, and hardly
/// show.
#[test]
fn meshes_are_drawn_over_the_background() {
    let id = ImageId::temp(ImageFormat::Png);
    let gold = PbrMaterial::new().color(carbide_core::color::rgba(1.0, 0.8, 0.4, 1.0)).metallic(1.0).roughness(Some(0.0f32));
    let scene = Scene3d::new(Object::new(Mesh::icosphere(1.0, 5), gold))
        .environment(Some(id.clone()))
        .background(Background::Environment { blur: 0.0 })
        .tonemapping(Tonemapping::None)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, 4.0), Point3::new(0.0, 0.0, 0.0)));
    let mut harness = rendered(scene, 200.0, &id, sky);

    let [r, g, b] = pixel(&mut harness, 100, 100);
    assert!(r > 200 && g < 80 && b < 80, "the sphere's middle: {:?}, not the square behind the camera", [r, g, b]);
    let [r, _, b] = pixel(&mut harness, 10, 10);
    assert!(b > r + 40, "the background's top: {:?}, not the sky", [r, b]);
    harness.snapshot("sphere");
}

/// Blurred, the background spreads the red square and softens the horizon: at a blur of 1, the
/// square's middle is far from red, and the horizon is between sky and ground. Sharp, the square
/// is red and the sky meets the ground within a few pixels.
#[test]
fn the_background_can_be_blurred() {
    let id = ImageId::temp(ImageFormat::Png);
    let scene = |blur: f32| Scene3d::new(())
        .environment(Some(id.clone()))
        .background(Background::Environment { blur })
        .tonemapping(Tonemapping::None)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 1.0)))
        .frame(200.0, 200.0);
    let mut harness = rendered(HStack::new((scene(0.0), scene(0.5), scene(1.0))).spacing(0.0), 600.0, &id, sky);

    let sharp = pixel(&mut harness, 100, 100);
    let blurred = pixel(&mut harness, 500, 100);
    assert!(sharp[0] > 200 && sharp[1] < 80, "sharp, the square is {sharp:?}");
    assert!(blurred[1] > 80, "blurred, the square is still {blurred:?}");

    // Down the left edge, 30 pixels from the horizon on either side.
    let above = |x: u32, harness: &mut Harness| pixel(harness, x, 70);
    let below = |x: u32, harness: &mut Harness| pixel(harness, x, 130);
    let sharp_contrast = above(30, &mut harness)[2] as i32 - below(30, &mut harness)[2] as i32;
    let blurred_contrast = above(430, &mut harness)[2] as i32 - below(430, &mut harness)[2] as i32;
    assert!(blurred_contrast < sharp_contrast - 40, "blurring doesn't soften the horizon: {sharp_contrast} against {blurred_contrast}");
    harness.snapshot("blurs");
}

/// While the scene has no environment loaded, an environment background is transparent: the view
/// looks as with a transparent colour.
#[test]
fn an_environment_background_without_an_environment_is_transparent() {
    let missing = ImageId::temp(ImageFormat::Png);
    let unused = ImageId::temp(ImageFormat::Png);
    let scene = |background: Background| sphere_scene(matte()).environment(Some(missing.clone())).background(background);
    let mut environment = rendered(scene(Background::Environment { blur: 0.3 }), 200.0, &unused, sky);
    let mut transparent = rendered(scene(Background::Color(carbide_core::color::TRANSPARENT)), 200.0, &unused, sky);
    assert!(environment.pixels().as_raw() == transparent.pixels().as_raw(), "the background isn't transparent");
}

/// Whether any channel of `a` and `b` differs by more than `tolerance`.
trait AbsDiffAny {
    fn abs_diff_any(&self, other: [u8; 4], tolerance: u8) -> bool;
}

impl AbsDiffAny for [u8; 4] {
    fn abs_diff_any(&self, other: [u8; 4], tolerance: u8) -> bool {
        self.iter().zip(other).any(|(a, b)| a.abs_diff(b) > tolerance)
    }
}
