//! Textures, shared with 2D through one image store.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Mesh, Object, Scene3d};
use carbide_3d::material::sample_type::SampleType;
use carbide_3d::Node3dExt;
use carbide_core::draw::{Dimension, ImageContext, ImageFormat, ImageId, Texture, TextureFormat};
use carbide_core::math::{Deg, Euler, Matrix3, Point3, Vector3};
use carbide_core::widget::{HStack, Image, WidgetExt};
use carbide_testing::Harness;
use carbide_wgpu_core::{WgpuImageContext, WgpuTextureStore};

mod common;

/// An image shown in an `Image` widget and used as a cube's albedo is uploaded once, into the
/// store both renderers draw from, and is not uploaded again on later frames.
#[test]
fn an_image_in_2d_and_3d_is_uploaded_once() {
    let rust = ImageId::new("images/rust.png");

    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(rust.clone()));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);
    let scene = Scene3d::new((cube, light))
        .camera(Camera::perspective().look_at(Point3::new(-2.0, 2.0, 3.0), Point3::new(0.0, 0.0, 0.0)))
        .frame(150.0, 150.0);

    let widget = HStack::new((Image::new(rust.clone()), scene));

    let mut harness = Harness::new(widget, Dimension::new(300.0, 150.0));
    harness.render();

    let textures = harness.environment().get::<WgpuTextureStore>().unwrap();
    assert_eq!(textures.len(), 1, "one stored texture for the image, used by 2D and 3D");
    let stored = textures.get(&rust).expect("the image is in the shared store");
    assert!(stored.width > 0 && stored.height > 0);
    let version = stored.version;

    harness.render();

    let textures = harness.environment().get::<WgpuTextureStore>().unwrap();
    assert_eq!(textures.len(), 1);
    assert_eq!(textures.get(&rust).unwrap().version, version, "not uploaded again");
}

/// A material's texture is loaded by the 3D scene alone, with no 2D widget showing the image.
#[test]
fn a_material_loads_its_texture_into_the_shared_store() {
    let rust = ImageId::new("images/rust.png");

    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(rust.clone()));
    let scene = Scene3d::new(cube)
        .camera(Camera::perspective().look_at(Point3::new(-2.0, 2.0, 3.0), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(150.0, 150.0));
    harness.render();

    let textures = harness.environment().get::<WgpuTextureStore>().unwrap();
    assert!(textures.contains(&rust));
}

/// A 64×64 checker of 8×8 squares, orange and white, as RGBA8.
fn checker() -> Vec<u8> {
    let mut data = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64 {
        for x in 0..64 {
            let orange = (x / 8 + y / 8) % 2 == 0;
            data.extend_from_slice(if orange { &[255, 128, 0, 255] } else { &[255, 255, 255, 255] });
        }
    }
    data
}

/// Store the checker under a new in-memory id, through the harness's image context.
fn store_checker(harness: &mut Harness, id: &ImageId) {
    let data = checker();
    let texture = Texture { width: 64, height: 64, bytes_per_row: 64 * 4, format: TextureFormat::RGBA8, data: &data };
    WgpuImageContext.update_texture(id, texture, harness.environment_mut());
}

/// A cube with a checker texture as its albedo, lit by one light. The first frame is drawn before
/// the image exists, so the cube is white; once the image is stored, the cube's bind group is made
/// again and the checker shows, each face covered once.
#[test]
fn a_cube_with_a_checker_texture() {
    let checker_id = ImageId::temp(ImageFormat::Png);

    let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(checker_id.clone()).roughness(Some(0.6f32)))
        .rotation(Euler::new(Deg(0.0), Deg(-30.0), Deg(0.0)));
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(5.0);
    let scene = Scene3d::new((cube, light))
        .camera(Camera::perspective().look_at(Point3::new(-1.5, 1.5, 2.5), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(240.0, 240.0));
    harness.snapshot("before_loading");

    store_checker(&mut harness, &checker_id);
    harness.snapshot("checker");
}

/// A large plane with the checker repeated 16 times across it, seen at a grazing angle. Covers
/// repeating texture coordinates, and mip levels and anisotropic filtering in the distance, where
/// the squares blend to the average colour instead of aliasing. Next to it, the same plane with
/// nearest sampling, which aliases.
#[test]
fn a_repeated_texture_in_the_distance() {
    let checker_id = ImageId::temp(ImageFormat::Png);

    let plane = |sample_type: SampleType| {
        let mut material = PbrMaterial::new().color(checker_id.clone()).roughness(Some(0.8f32));
        material.uv_transform0 = Matrix3::from_scale(16.0);
        material.sample_type = sample_type;

        let light = DirectionalLight::new(Vector3::new(0.3, -1.0, -0.4)).intensity(4.0);
        Scene3d::new((Object::new(Mesh::plane(20.0, 20.0), material), light))
            .camera(Camera::perspective().look_at(Point3::new(0.0, 0.6, 3.0), Point3::new(0.0, 0.0, -2.0)))
            .frame(200.0, 150.0)
    };

    let widget = HStack::new((plane(SampleType::Linear), plane(SampleType::Nearest))).spacing(10.0);

    let mut harness = Harness::new(widget, Dimension::new(410.0, 150.0));
    store_checker(&mut harness, &checker_id);
    harness.snapshot("linear_and_nearest");
}

/// The checker on an icosphere, from the front and from the back, both from above to show the
/// top pole. The squares run evenly around and from pole to pole, with no smeared strip at the
/// seam behind the sphere and no twist at the pole.
#[test]
fn a_textured_icosphere() {
    let checker_id = ImageId::temp(ImageFormat::Png);

    let view = |eye: Point3<f32>| {
        let sphere = Object::new(Mesh::icosphere(1.0, 4), PbrMaterial::new().color(checker_id.clone()).roughness(Some(0.6f32)));
        let light = DirectionalLight::new(Vector3::new(0.5, -1.0, -1.0)).intensity(4.0);
        Scene3d::new((sphere, light))
            .camera(Camera::perspective().look_at(eye, Point3::new(0.0, 0.0, 0.0)))
            .frame(200.0, 200.0)
    };

    let widget = HStack::new((
        view(Point3::new(0.0, 2.0, 3.0)),
        view(Point3::new(0.0, 2.0, -3.0)),
    )).spacing(10.0);

    let mut harness = Harness::new(widget, Dimension::new(410.0, 200.0));
    store_checker(&mut harness, &checker_id);
    harness.snapshot("front_and_back");
}
