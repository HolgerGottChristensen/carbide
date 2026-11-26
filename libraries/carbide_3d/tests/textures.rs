//! Textures, shared with 2D through one image store.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Mesh, Object, Scene3d};
use carbide_3d::material::ao_mr_textures::AoMRTextures;
use carbide_3d::material::clearcoat_textures::ClearcoatTextures;
use carbide_3d::material::normal_texture::NormalTexture;
use carbide_3d::material::normal_texture_y_direction::NormalTextureYDirection;
use carbide_3d::material::sample_type::SampleType;
use carbide_3d::Node3dExt;
use carbide_core::color::WHITE;
use carbide_core::draw::Color;
use carbide_core::state::{LocalState, State};
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

/// A 4×4 image of one colour, stored under a new in-memory id.
fn store_flat_image(harness: &mut Harness, rgba: [u8; 4]) -> ImageId {
    let id = ImageId::temp(ImageFormat::Png);
    let data: Vec<u8> = rgba.iter().copied().cycle().take(4 * 4 * 4).collect();
    let texture = Texture { width: 4, height: 4, bytes_per_row: 16, format: TextureFormat::RGBA8, data: &data };
    WgpuImageContext.update_texture(&id, texture, harness.environment_mut());
    id
}

/// The brightness of the middle of a plane seen from straight above, with the normal map `map`
/// (a constant normal, as RGB) read with `direction`, lit by a light travelling along `light`.
/// `None` is the plane without a normal map.
fn plane_brightness(map: Option<([u8; 3], NormalTextureYDirection)>, light: Vector3<f32>) -> u32 {
    let normal = LocalState::new(NormalTexture::None);
    let plane = Object::new(Mesh::plane(4.0, 4.0), PbrMaterial::new().color(WHITE).roughness(Some(1.0f32)).normal(normal.clone()));
    let scene = Scene3d::new((plane, DirectionalLight::new(light).intensity(3.0)))
        .ambient(WHITE, 0.0)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 3.0, 0.001), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(60.0, 60.0));
    if let Some((rgb, direction)) = map {
        let id = store_flat_image(&mut harness, [rgb[0], rgb[1], rgb[2], 255]);
        *normal.clone().value_mut() = NormalTexture::Tricomponent(id, direction);
    }
    harness.render();

    let pixel = harness.pixels().get_pixel(30, 30).0;
    pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32
}

/// Normal maps bend the normal the way their colours say: red along the texture's right (+u),
/// green towards the image's top, which on the plane is the far edge (-Z), and the other way for
/// DirectX maps. A map tilted towards the light makes the surface brighter than a flat one, and
/// tilted away, darker.
#[test]
fn a_normal_map_bends_the_normal() {
    const UP: NormalTextureYDirection = NormalTextureYDirection::Up;
    const DOWN: NormalTextureYDirection = NormalTextureYDirection::Down;
    // Tilted 30° towards the image's top (green), and towards its right (red).
    let towards_top = [128, 191, 238];
    let towards_right = [191, 128, 238];

    // Light from the far edge (travelling towards +Z), and from the right (towards -X).
    let from_far = Vector3::new(0.0, -1.0, 1.0);
    let from_right = Vector3::new(-1.0, -1.0, 0.0);

    let flat = plane_brightness(None, from_far);
    let flat_map = plane_brightness(Some(([128, 128, 255], UP)), from_far);
    assert!(flat_map.abs_diff(flat) <= 6, "a flat normal map changes the plane: {flat_map} against {flat}");

    let top_opengl = plane_brightness(Some((towards_top, UP)), from_far);
    let top_directx = plane_brightness(Some((towards_top, DOWN)), from_far);
    assert!(top_opengl > flat + 20, "tilted towards the light, but not brighter: {top_opengl} against {flat}");
    assert!(top_directx + 20 < flat, "a DirectX map tilted away from the light, but not darker: {top_directx} against {flat}");

    let flat = plane_brightness(None, from_right);
    let right = plane_brightness(Some((towards_right, UP)), from_right);
    let right_directx = plane_brightness(Some((towards_right, DOWN)), from_right);
    assert!(right > flat + 20, "tilted towards the light, but not brighter: {right} against {flat}");
    assert!(right.abs_diff(right_directx) <= 6, "the DirectX flag changes red: {right} against {right_directx}");
}

/// A 64×64 normal map of 4×4 domes, green up (OpenGL): each dome's normal tilts outwards from its
/// centre, and the space between them is flat.
fn store_dome_normal_map(harness: &mut Harness) -> ImageId {
    let mut data = Vec::with_capacity(64 * 64 * 4);
    for row in 0..64 {
        for column in 0..64 {
            // Position in the cell, -1 to 1, with y towards the image's top.
            let x = ((column % 16) as f32 + 0.5) / 8.0 - 1.0;
            let y = 1.0 - ((row % 16) as f32 + 0.5) / 8.0;
            let r2 = x * x + y * y;

            let normal = if r2 < 0.64 {
                // A sphere cap: the normal is the direction from the dome's centre.
                let (nx, ny) = (x / 0.8 * 0.7, y / 0.8 * 0.7);
                Vector3::new(nx, ny, (1.0 - nx * nx - ny * ny).sqrt())
            } else {
                Vector3::new(0.0, 0.0, 1.0)
            };

            let encode = |value: f32| ((value * 0.5 + 0.5) * 255.0).round() as u8;
            data.extend_from_slice(&[encode(normal.x), encode(normal.y), encode(normal.z), 255]);
        }
    }

    let id = ImageId::temp(ImageFormat::Png);
    let texture = Texture { width: 64, height: 64, bytes_per_row: 64 * 4, format: TextureFormat::RGBA8, data: &data };
    WgpuImageContext.update_texture(&id, texture, harness.environment_mut());
    id
}

/// The dome normal map on a plane, lit from the far left, and repeated 8 × 4 times around an
/// icosphere, lit from the front left. Each dome is lit on its side towards the light and shaded on
/// the other, on the flat plane and around the sphere.
#[test]
fn a_normal_mapped_plane_and_sphere() {
    let normal = LocalState::new(NormalTexture::None);
    let material = || PbrMaterial::new().color(WHITE).roughness(Some(0.7f32)).normal(normal.clone());

    let plane = Scene3d::new((
        Object::new(Mesh::plane(2.0, 2.0), material()),
        DirectionalLight::new(Vector3::new(1.0, -1.0, 1.0)).intensity(4.0),
    ))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.2, 0.8), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut sphere_material = material();
    sphere_material.uv_transform0 = Matrix3::from_nonuniform_scale(8.0, 4.0);
    let sphere = Scene3d::new((
        Object::new(Mesh::icosphere(1.0, 4), sphere_material),
        DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0)).intensity(4.0),
    ))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.5, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((plane, sphere)).spacing(10.0), Dimension::new(410.0, 200.0));
    let id = store_dome_normal_map(&mut harness);
    *normal.clone().value_mut() = NormalTexture::Tricomponent(id, NormalTextureYDirection::Up);
    harness.snapshot("domes");
}

/// How a test plane is lit.
#[derive(Clone, Copy)]
enum Lighting {
    /// A light straight down onto the plane, seen from straight above: the highlight is in the
    /// middle, where roughness and metallic show. No ambient light.
    Highlight,
    /// Ambient light only, which occlusion darkens.
    Ambient,
    /// No light at all: only the surface's own light shows.
    Dark,
}

/// The brightness of the middle of a grey plane seen from straight above. `maps` stores the
/// images it needs and gives the material's maps; `roughness` and `metallic` are the material's
/// factors.
fn aomr_brightness(lighting: Lighting, roughness: f32, metallic: f32, occlusion: f32, maps: impl FnOnce(&mut Harness) -> AoMRTextures) -> u32 {
    let textures = LocalState::new(AoMRTextures::None);
    let material = PbrMaterial::new()
        .color(Color::new_rgba(120, 120, 120, 255))
        .roughness(Some(roughness))
        .metallic(metallic)
        .ambient_occlusion(occlusion)
        .aomr_textures(textures.clone());

    let plane = Object::new(Mesh::plane(4.0, 4.0), material);
    let camera = Camera::perspective().look_at(Point3::new(0.0, 3.0, 0.001), Point3::new(0.0, 0.0, 0.0));
    // The light, off for ambient lighting, and the ambient light, off for the highlight.
    let (light, ambient) = match lighting {
        Lighting::Highlight => (3.0, 0.0),
        Lighting::Ambient => (0.0, 1.0),
        Lighting::Dark => (0.0, 0.0),
    };
    let scene = Scene3d::new((plane, DirectionalLight::new(Vector3::new(0.0, -1.0, 0.0)).intensity(light)))
        .ambient(WHITE, ambient)
        .camera(camera);

    let mut harness = Harness::new(scene, Dimension::new(60.0, 60.0));
    *textures.clone().value_mut() = maps(&mut harness);
    harness.render();

    let pixel = harness.pixels().get_pixel(30, 30).0;
    pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32
}

/// Assert that a map of `low` and `high` (as bytes) renders as the factors `low / 255` and
/// `high / 255` do without a map, and that the two differ.
fn assert_map_matches_factor(name: &str, lighting: Lighting, value: impl Fn(f32) -> (f32, f32, f32), maps: impl Fn(&mut Harness, u8) -> AoMRTextures) {
    let (low, high) = (64u8, 230u8);
    let factor = |byte: u8| byte as f32 / 255.0;

    let plain = |byte: u8| {
        let (roughness, metallic, occlusion) = value(factor(byte));
        aomr_brightness(lighting, roughness, metallic, occlusion, |_| AoMRTextures::None)
    };
    let mapped = |byte: u8| {
        let (roughness, metallic, occlusion) = value(1.0);
        aomr_brightness(lighting, roughness, metallic, occlusion, |harness| maps(harness, byte))
    };

    let (plain_low, plain_high) = (plain(low), plain(high));
    assert!(plain_low.abs_diff(plain_high) > 20, "{name}: the factor makes no difference: {plain_low} and {plain_high}");

    for byte in [low, high] {
        let (with_map, without) = (mapped(byte), plain(byte));
        assert!(with_map.abs_diff(without) <= 6, "{name}: a map of {byte} gives {with_map}, the factor alone {without}");
    }
}

/// Roughness, metallic and occlusion maps, in each of the layouts `AoMRTextures` has, multiply the
/// material's factors by the right channel: a map of a value renders as that factor does.
#[test]
fn occlusion_metallic_and_roughness_maps() {
    use Lighting::{Ambient, Highlight};
    // The material's (roughness, metallic, occlusion) for a value of the channel under test.
    let roughness = |v: f32| (v, 0.0, 1.0);
    let metallic = |v: f32| (0.5, v, 1.0);
    let occlusion = |v: f32| (0.5, 0.0, v);

    let image = |harness: &mut Harness, rgb: [u8; 3]| Some(store_flat_image(harness, [rgb[0], rgb[1], rgb[2], 255]));

    // Combined: occlusion, roughness and metallic in red, green and blue of one map.
    assert_map_matches_factor("combined roughness", Highlight, roughness, |h, b| AoMRTextures::Combined { texture: image(h, [255, b, 255]) });
    assert_map_matches_factor("combined metallic", Highlight, metallic, |h, b| AoMRTextures::Combined { texture: image(h, [255, 255, b]) });
    assert_map_matches_factor("combined occlusion", Ambient, occlusion, |h, b| AoMRTextures::Combined { texture: image(h, [b, 255, 255]) });

    // Split: roughness and metallic in red and green; occlusion in its own map's red.
    assert_map_matches_factor("split roughness", Highlight, roughness, |h, b| AoMRTextures::Split { ao_texture: None, mr_texture: image(h, [b, 255, 0]) });
    assert_map_matches_factor("split metallic", Highlight, metallic, |h, b| AoMRTextures::Split { ao_texture: None, mr_texture: image(h, [255, b, 0]) });
    assert_map_matches_factor("split occlusion", Ambient, occlusion, |h, b| AoMRTextures::Split { ao_texture: image(h, [b, 0, 0]), mr_texture: None });

    // Swizzled split: roughness and metallic in green and blue.
    assert_map_matches_factor("swizzled roughness", Highlight, roughness, |h, b| AoMRTextures::SwizzledSplit { ao_texture: None, mr_texture: image(h, [0, b, 255]) });
    assert_map_matches_factor("swizzled metallic", Highlight, metallic, |h, b| AoMRTextures::SwizzledSplit { ao_texture: None, mr_texture: image(h, [0, 255, b]) });

    // Greyscale maps, each read from red.
    assert_map_matches_factor("greyscale roughness", Highlight, roughness, |h, b| AoMRTextures::BWSplit { ao_texture: None, m_texture: None, r_texture: image(h, [b, b, b]) });
    assert_map_matches_factor("greyscale metallic", Highlight, metallic, |h, b| AoMRTextures::BWSplit { ao_texture: None, m_texture: image(h, [b, b, b]), r_texture: None });
    assert_map_matches_factor("greyscale occlusion", Ambient, occlusion, |h, b| AoMRTextures::BWSplit { ao_texture: image(h, [b, b, b]), m_texture: None, r_texture: None });
}

/// A roughness map with the material's roughness unset is used as it is, as glTF's factor of 1,
/// not multiplied by the default of 0, which would make it a mirror.
#[test]
fn an_unset_roughness_uses_the_map() {
    let render = |roughness: Option<f32>, map: Option<u8>| {
        let textures = LocalState::new(AoMRTextures::None);
        let mut material = PbrMaterial::new().color(Color::new_rgba(120, 120, 120, 255)).aomr_textures(textures.clone());
        if let Some(roughness) = roughness {
            material = material.roughness(Some(roughness));
        }
        let scene = Scene3d::new((Object::new(Mesh::plane(4.0, 4.0), material), DirectionalLight::new(Vector3::new(0.0, -1.0, 0.0)).intensity(3.0)))
            .ambient(WHITE, 0.0)
            .camera(Camera::perspective().look_at(Point3::new(0.0, 3.0, 0.001), Point3::new(0.0, 0.0, 0.0)));

        let mut harness = Harness::new(scene, Dimension::new(60.0, 60.0));
        if let Some(byte) = map {
            let id = store_flat_image(&mut harness, [byte, byte, byte, 255]);
            *textures.clone().value_mut() = AoMRTextures::BWSplit { ao_texture: None, m_texture: None, r_texture: Some(id) };
        }
        harness.render();
        let pixel = harness.pixels().get_pixel(30, 30).0;
        pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32
    };

    let unset_with_map = render(None, Some(204));
    let set_to_map_value = render(Some(0.8), None);
    let mirror = render(Some(0.0), None);
    assert!(unset_with_map.abs_diff(set_to_map_value) <= 6, "unset roughness with a map of 0.8: {unset_with_map}, roughness 0.8: {set_to_map_value}");
    assert!(unset_with_map.abs_diff(mirror) > 20, "looks like a mirror: {unset_with_map} against {mirror}");
}

/// A 64×64 image of eight vertical stripes, alternating between `a` and `b` in every channel.
fn store_stripes(harness: &mut Harness, a: u8, b: u8) -> ImageId {
    let mut data = Vec::with_capacity(64 * 64 * 4);
    for _row in 0..64 {
        for column in 0..64 {
            let value = if (column / 8) % 2 == 0 { a } else { b };
            data.extend_from_slice(&[value, value, value, 255]);
        }
    }
    let id = ImageId::temp(ImageFormat::Png);
    let texture = Texture { width: 64, height: 64, bytes_per_row: 64 * 4, format: TextureFormat::RGBA8, data: &data };
    WgpuImageContext.update_texture(&id, texture, harness.environment_mut());
    id
}

/// Left: a sphere with a roughness map of glossy (0.3) and matte (0.9) stripes running from pole
/// to pole, 24 around, lit from the front left: the highlight shows on the glossy stripes and
/// fades on the matte ones. Right: a plane under ambient light only, with stripes of 0.25 and 1 as
/// its occlusion map: the occluded stripes are darker.
#[test]
fn roughness_and_occlusion_maps() {
    let roughness = LocalState::new(AoMRTextures::None);
    let occlusion = LocalState::new(AoMRTextures::None);

    let mut sphere_material = PbrMaterial::new().color(Color::new_rgba(40, 90, 200, 255)).aomr_textures(roughness.clone());
    sphere_material.uv_transform0 = Matrix3::from_nonuniform_scale(3.0, 1.0);
    let sphere = Scene3d::new((
        Object::new(Mesh::icosphere(1.0, 4), sphere_material),
        DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0)).intensity(4.0),
    ))
        .ambient(WHITE, 0.2)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let plane = Scene3d::new(Object::new(Mesh::plane(2.0, 2.0), PbrMaterial::new().color(Color::new_rgba(200, 200, 200, 255)).aomr_textures(occlusion.clone())))
        .ambient(WHITE, 1.0)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.2, 0.8), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((sphere, plane)).spacing(10.0), Dimension::new(410.0, 200.0));

    let roughness_stripes = store_stripes(&mut harness, 77, 230);
    *roughness.clone().value_mut() = AoMRTextures::BWSplit { ao_texture: None, m_texture: None, r_texture: Some(roughness_stripes) };

    let occlusion_stripes = store_stripes(&mut harness, 64, 255);
    *occlusion.clone().value_mut() = AoMRTextures::BWSplit { ao_texture: Some(occlusion_stripes), m_texture: None, r_texture: None };

    harness.snapshot("stripes");
}

/// The brightness of the middle of a plane with `material`, seen from straight above, after
/// `setup` has stored its images and set its states.
fn material_brightness(lighting: Lighting, material: PbrMaterial, setup: impl FnOnce(&mut Harness)) -> u32 {
    let (light, ambient) = match lighting {
        Lighting::Highlight => (3.0, 0.0),
        Lighting::Ambient => (0.0, 1.0),
        Lighting::Dark => (0.0, 0.0),
    };
    let scene = Scene3d::new((Object::new(Mesh::plane(4.0, 4.0), material), DirectionalLight::new(Vector3::new(0.0, -1.0, 0.0)).intensity(light)))
        .ambient(WHITE, ambient)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 3.0, 0.001), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(60.0, 60.0));
    setup(&mut harness);
    harness.render();

    let pixel = harness.pixels().get_pixel(30, 30).0;
    pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32
}

/// Assert that a clear-coat map of a value renders as the clear coat's factor of that value does
/// without a map, for a low and a high value that render differently. `factors` gives the clear
/// coat's (strength, roughness) for a value of the channel under test.
fn assert_clearcoat_map(name: &str, factors: impl Fn(f32) -> (f32, f32), maps: impl Fn(&mut Harness, u8) -> ClearcoatTextures) {
    let (low, high) = (64u8, 230u8);
    let base = |strength: f32, roughness: f32, textures: LocalState<ClearcoatTextures>| {
        PbrMaterial::new().color(Color::new_rgba(120, 120, 120, 255)).roughness(Some(0.1f32)).clearcoat(strength, roughness).clearcoat_textures(textures)
    };

    let plain = |byte: u8| {
        let (strength, roughness) = factors(byte as f32 / 255.0);
        material_brightness(Lighting::Highlight, base(strength, roughness, LocalState::new(ClearcoatTextures::None)), |_| {})
    };
    let mapped = |byte: u8| {
        let (strength, roughness) = factors(1.0);
        let textures = LocalState::new(ClearcoatTextures::None);
        material_brightness(Lighting::Highlight, base(strength, roughness, textures.clone()), |harness| {
            *textures.clone().value_mut() = maps(harness, byte);
        })
    };

    let (plain_low, plain_high) = (plain(low), plain(high));
    assert!(plain_low.abs_diff(plain_high) > 20, "{name}: the factor makes no difference: {plain_low} and {plain_high}");
    for byte in [low, high] {
        let (with_map, without) = (mapped(byte), plain(byte));
        assert!(with_map.abs_diff(without) <= 6, "{name}: a map of {byte} gives {with_map}, the factor alone {without}");
    }
}

/// Clear-coat maps, in each layout `ClearcoatTextures` has, multiply the clear coat's strength and
/// roughness by the right channel. For now the clear coat only widens the highlight towards its
/// roughness, so the base is glossy and the coat rough.
#[test]
fn clear_coat_maps() {
    let image = |harness: &mut Harness, rgb: [u8; 3]| Some(store_flat_image(harness, [rgb[0], rgb[1], rgb[2], 255]));
    let strength = |v: f32| (v, 0.9);
    let roughness = |v: f32| (1.0, v);

    // Combined: strength in red, roughness in green of one map.
    assert_clearcoat_map("combined strength", strength, |h, b| ClearcoatTextures::GltfCombined { texture: image(h, [b, 255, 0]) });
    assert_clearcoat_map("combined roughness", roughness, |h, b| ClearcoatTextures::GltfCombined { texture: image(h, [255, b, 0]) });

    // glTF's split: strength in its map's red, roughness in its map's green.
    assert_clearcoat_map("split strength", strength, |h, b| ClearcoatTextures::GltfSplit { clearcoat_texture: image(h, [b, 0, 0]), clearcoat_roughness_texture: None });
    assert_clearcoat_map("split roughness", roughness, |h, b| ClearcoatTextures::GltfSplit { clearcoat_texture: None, clearcoat_roughness_texture: image(h, [0, b, 0]) });

    // Greyscale: both from red.
    assert_clearcoat_map("greyscale strength", strength, |h, b| ClearcoatTextures::BWSplit { clearcoat_texture: image(h, [b, b, b]), clearcoat_roughness_texture: None });
    assert_clearcoat_map("greyscale roughness", roughness, |h, b| ClearcoatTextures::BWSplit { clearcoat_texture: None, clearcoat_roughness_texture: image(h, [b, b, b]) });
}

/// An emission map multiplies the emissive colour, read as a colour (sRGB): in the dark, a grey
/// map with a white emissive colour glows as that grey emissive colour does without a map, and
/// a map without an emissive colour gives off nothing.
#[test]
fn an_emission_map() {
    let black = Color::new_rgba(0, 0, 0, 255);
    let grey = |byte: u8| Color::new_rgba(byte, byte, byte, 255);

    let plain = |byte: u8| material_brightness(Lighting::Dark, PbrMaterial::new().color(black).emissive(grey(byte)), |_| {});
    let mapped = |byte: u8, emissive: Color| {
        let map = LocalState::new(None::<ImageId>);
        material_brightness(Lighting::Dark, PbrMaterial::new().color(black).emissive(emissive).emissive_map(map.clone()), |harness| {
            *map.clone().value_mut() = Some(store_flat_image(harness, [byte, byte, byte, 255]));
        })
    };

    let (plain_low, plain_high) = (plain(64), plain(230));
    assert!(plain_high > plain_low + 100, "the emissive colour makes little difference: {plain_low} and {plain_high}");
    for byte in [64u8, 230] {
        let (with_map, without) = (mapped(byte, WHITE), plain(byte));
        assert!(with_map.abs_diff(without) <= 6, "a map of {byte} glows {with_map}, the colour alone {without}");
    }
    assert!(mapped(230, black) <= 3, "a map with a black emissive colour glows");
}

/// Left: a dark sphere with the checker as its emission map and a white emissive colour, in dim
/// light, so the checker glows by itself. Right: a glossy blue sphere with stripes of rough clear
/// coat (strength 0 and 1), lit from the front left, so the highlight is sharp between the
/// stripes and spread on them.
#[test]
fn emission_and_clear_coat_maps() {
    let emission = LocalState::new(None::<ImageId>);
    let coat = LocalState::new(ClearcoatTextures::None);

    let glowing = Scene3d::new(Object::new(
        Mesh::icosphere(1.0, 4),
        PbrMaterial::new().color(Color::new_rgba(30, 30, 30, 255)).roughness(Some(0.8f32)).emissive(WHITE).emissive_map(emission.clone()),
    ))
        .ambient(WHITE, 0.1)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut coated_material = PbrMaterial::new()
        .color(Color::new_rgba(40, 90, 200, 255))
        .roughness(Some(0.2f32))
        .clearcoat(1.0, 0.8)
        .clearcoat_textures(coat.clone());
    coated_material.uv_transform0 = Matrix3::from_nonuniform_scale(3.0, 1.0);
    let coated = Scene3d::new((
        Object::new(Mesh::icosphere(1.0, 4), coated_material),
        DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0)).intensity(4.0),
    ))
        .ambient(WHITE, 0.2)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((glowing, coated)).spacing(10.0), Dimension::new(410.0, 200.0));

    let checker_id = ImageId::temp(ImageFormat::Png);
    store_checker(&mut harness, &checker_id);
    *emission.clone().value_mut() = Some(checker_id);

    let stripes = store_stripes(&mut harness, 0, 255);
    *coat.clone().value_mut() = ClearcoatTextures::BWSplit { clearcoat_texture: Some(stripes), clearcoat_roughness_texture: None };

    harness.snapshot("glow_and_coat");
}
