//! Textures, shared with 2D through one image store.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Mesh, Object, Scene3d};
use carbide_core::draw::{Dimension, ImageId};
use carbide_core::math::{Point3, Vector3};
use carbide_core::widget::{HStack, Image, WidgetExt};
use carbide_testing::Harness;
use carbide_wgpu_core::WgpuTextureStore;

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
