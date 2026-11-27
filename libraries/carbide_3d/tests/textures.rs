//! Textures, shared with 2D through one image store.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Mesh, Object, Scene3d};
use carbide_3d::material::ao_mr_textures::AoMRTextures;
use carbide_3d::material::clearcoat_textures::ClearcoatTextures;
use carbide_3d::material::material_texture::MaterialTexture;
use carbide_3d::material::normal_texture::NormalTexture;
use carbide_3d::material::normal_texture_y_direction::NormalTextureYDirection;
use carbide_3d::material::sample_type::SampleType;
use carbide_3d::Node3dExt;
use carbide_core::color::WHITE;
use carbide_core::draw::Color;
use carbide_core::lens;
use carbide_core::state::{IntoReadState, LocalState, Map1, Map2, State};
use carbide_3d::material::Material;
use carbide_core::draw::{Dimension, ImageContext, ImageFormat, ImageId, Texture, TextureFormat};
use carbide_core::math::{Deg, Euler, Point3, Rad, Vector2, Vector3};
use carbide_core::widget::{HStack, Image, Rectangle, Widget, WidgetExt};
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
        let material = PbrMaterial::new()
            .color(checker_id.clone())
            .roughness(Some(0.8f32))
            .tiling(Vector2::new(16.0, 16.0))
            .sample_type(sample_type);

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
    mesh_brightness(Mesh::plane(4.0, 4.0), map, 1.0, light)
}

/// [`plane_brightness`] for any `mesh` lying on the ground and facing up, with the material's
/// normal scale `scale`.
fn mesh_brightness(mesh: Mesh, map: Option<([u8; 3], NormalTextureYDirection)>, scale: f32, light: Vector3<f32>) -> u32 {
    let material = LocalState::new(PbrMaterial::new().color(WHITE).roughness(1.0).normal_scale(scale));
    let normal = lens!(material.normal);
    let plane = Object::new(mesh, material.clone());
    let scene = Scene3d::new((plane, DirectionalLight::new(light).intensity(3.0)))
        .ambient(WHITE, 0.0)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 3.0, 0.001), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(60.0, 60.0));
    if let Some((rgb, direction)) = map {
        let id = store_flat_image(&mut harness, [rgb[0], rgb[1], rgb[2], 255]);
        *normal.clone().value_mut() = NormalTexture::Tricomponent(id.into(), direction);
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
    // One normal map, set once the image is stored, shared by every material made here.
    let normal = LocalState::new(NormalTexture::None);
    let tiled = |tiling: Vector2<f32>| Map1::read_map(normal.clone(), move |normal: &NormalTexture| {
        PbrMaterial::new().color(WHITE).roughness(0.7).normal(normal.clone()).tiling(tiling)
    });
    let material = || tiled(Vector2::new(1.0, 1.0));

    let plane = Scene3d::new((
        Object::new(Mesh::plane(2.0, 2.0), material()),
        DirectionalLight::new(Vector3::new(1.0, -1.0, 1.0)).intensity(4.0),
    ))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.2, 0.8), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let sphere_material = tiled(Vector2::new(8.0, 4.0));
    let sphere = Scene3d::new((
        Object::new(Mesh::icosphere(1.0, 4), sphere_material),
        DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0)).intensity(4.0),
    ))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.5, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((plane, sphere)).spacing(10.0), Dimension::new(410.0, 200.0));
    let id = store_dome_normal_map(&mut harness);
    *normal.clone().value_mut() = NormalTexture::Tricomponent(id.into(), NormalTextureYDirection::Up);
    harness.snapshot("domes");
}

/// `mesh` with its texture mirrored left to right (u becomes 1 - u), and its tangents calculated
/// again.
fn mirrored(mesh: Mesh) -> Mesh {
    let mut vertices = mesh.vertices().to_vec();
    for vertex in &mut vertices {
        vertex.texture_coords_0.x = 1.0 - vertex.texture_coords_0.x;
    }
    Mesh::new(vertices, mesh.indices().to_vec()).calculate_tangents()
}

/// A plane whose texture is mirrored left to right has its tangent pointing the other way, and
/// the tangent's sign keeps the image's top at the far edge: a map tilted towards the image's top
/// is lit by a light from the far edge as on the plane that isn't mirrored. Without the sign, the
/// mirrored plane would tilt towards the near edge and be darker.
#[test]
fn a_mirrored_texture_keeps_its_normal_map_the_right_way_up() {
    const UP: NormalTextureYDirection = NormalTextureYDirection::Up;
    let towards_top = [128, 191, 238];
    let from_far = Vector3::new(0.0, -1.0, 1.0);

    let flat = plane_brightness(None, from_far);
    let plain = plane_brightness(Some((towards_top, UP)), from_far);
    let mirrored = mesh_brightness(mirrored(Mesh::plane(4.0, 4.0)), Some((towards_top, UP)), 1.0, from_far);
    assert!(plain > flat + 20, "tilted towards the light, but not brighter: {plain} against {flat}");
    assert!(mirrored.abs_diff(plain) <= 6, "the mirrored plane differs: {mirrored} against {plain}");
}

/// The normal scale multiplies the map's sideways components before normalising, as glTF's
/// `normalTexture.scale`: a map tilted 30° away from the light, at a scale of 0.5, renders as a
/// map tilted atan(0.5 · tan 30°) ≈ 16°; at 0 it is flat; at 2 (49°) it is darker than at 1.
#[test]
fn the_normal_scale_steepens_or_flattens_the_map() {
    const UP: NormalTextureYDirection = NormalTextureYDirection::Up;
    let plane = || Mesh::plane(4.0, 4.0);
    // Tilted towards the image's bottom, the near edge, away from a light from the far edge.
    let away_30 = [128, 64, 238];
    let away_16 = [128, 93, 245];
    let from_far = Vector3::new(0.0, -1.0, 1.0);

    let flat = plane_brightness(None, from_far);
    let full = mesh_brightness(plane(), Some((away_30, UP)), 1.0, from_far);
    let half = mesh_brightness(plane(), Some((away_30, UP)), 0.5, from_far);
    let none = mesh_brightness(plane(), Some((away_30, UP)), 0.0, from_far);
    let double = mesh_brightness(plane(), Some((away_30, UP)), 2.0, from_far);
    let shallower_map = plane_brightness(Some((away_16, UP)), from_far);

    assert!(full + 20 < flat, "tilted away from the light, but not darker: {full} against {flat}");
    assert!(none.abs_diff(flat) <= 6, "a scale of 0 isn't flat: {none} against {flat}");
    assert!(half.abs_diff(shallower_map) <= 6, "a scale of 0.5: {half}, the 16° map: {shallower_map}");
    assert!(double + 20 < full, "a scale of 2 isn't steeper: {double} against {full}");
}

/// The dome normal map on two planes side by side, the right one with its texture mirrored, lit
/// from the far edge: every dome is lit on its far side, on both planes.
#[test]
fn a_normal_map_on_a_mirrored_texture() {
    // One normal map, set once the image is stored, shared by every material made here.
    let normal = LocalState::new(NormalTexture::None);
    let tiled = |tiling: Vector2<f32>| Map1::read_map(normal.clone(), move |normal: &NormalTexture| {
        PbrMaterial::new().color(WHITE).roughness(0.7).normal(normal.clone()).tiling(tiling)
    });
    let material = || tiled(Vector2::new(1.0, 1.0));

    let scene = Scene3d::new((
        Object::new(Mesh::plane(1.0, 1.0), material()).position(Vector3::new(-0.52, 0.0, 0.0)),
        Object::new(mirrored(Mesh::plane(1.0, 1.0)), material()).position(Vector3::new(0.52, 0.0, 0.0)),
        DirectionalLight::new(Vector3::new(0.0, -1.0, 1.0)).intensity(4.0),
    ))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.0, 0.6), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(300.0, 160.0));
    let id = store_dome_normal_map(&mut harness);
    *normal.clone().value_mut() = NormalTexture::Tricomponent(id.into(), NormalTextureYDirection::Up);
    harness.snapshot("domes_mirrored");
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
    aomr_brightness_with(lighting, roughness, metallic, occlusion, 1.0, maps)
}

/// [`aomr_brightness`] with the material's occlusion strength `strength`.
fn aomr_brightness_with(lighting: Lighting, roughness: f32, metallic: f32, occlusion: f32, strength: f32, maps: impl FnOnce(&mut Harness) -> AoMRTextures) -> u32 {
    let material = LocalState::new(PbrMaterial::new()
        .occlusion_strength(strength)
        .color(Color::new_rgba(120, 120, 120, 255))
        .roughness(roughness)
        .metallic(metallic)
        .ambient_occlusion(occlusion));
    let textures = lens!(material.aomr_textures);

    let plane = Object::new(Mesh::plane(4.0, 4.0), material.clone());
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

    let image = |harness: &mut Harness, rgb: [u8; 3]| Some(store_flat_image(harness, [rgb[0], rgb[1], rgb[2], 255]).into());

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

/// The occlusion strength weakens the occlusion map, as glTF's `occlusionTexture.strength`: a map of
/// 0.25 at a strength of 0.5 renders as an occlusion of 1 + 0.5 · (0.25 − 1) = 0.625, and at 0 as no
/// occlusion. It doesn't weaken the material's own `ambient_occlusion`.
#[test]
fn the_occlusion_strength_weakens_the_map() {
    let map = |byte: u8| move |harness: &mut Harness| AoMRTextures::BWSplit {
        ao_texture: Some(store_flat_image(harness, [byte, byte, byte, 255]).into()),
        m_texture: None,
        r_texture: None,
    };
    let none = |_: &mut Harness| AoMRTextures::None;

    let half = aomr_brightness_with(Lighting::Ambient, 0.5, 0.0, 1.0, 0.5, map(64));
    let factor = aomr_brightness(Lighting::Ambient, 0.5, 0.0, 1.0 + 0.5 * (64.0 / 255.0 - 1.0), none);
    let full = aomr_brightness(Lighting::Ambient, 0.5, 0.0, 1.0, map(64));
    let off = aomr_brightness_with(Lighting::Ambient, 0.5, 0.0, 1.0, 0.0, map(64));
    let unoccluded = aomr_brightness(Lighting::Ambient, 0.5, 0.0, 1.0, none);
    let factor_only = aomr_brightness_with(Lighting::Ambient, 0.5, 0.0, 0.25, 0.0, none);
    let factor_quarter = aomr_brightness(Lighting::Ambient, 0.5, 0.0, 0.25, none);

    assert!(full + 20 < half, "a strength of 0.5 doesn't weaken the map: {half} against {full}");
    assert!(half.abs_diff(factor) <= 6, "a strength of 0.5: {half}, the factor 0.625: {factor}");
    assert!(off.abs_diff(unoccluded) <= 6, "a strength of 0 still occludes: {off} against {unoccluded}");
    assert_eq!(factor_only, factor_quarter, "the strength changes the material's own occlusion");
}

/// A roughness map with the material's roughness unset is used as it is, as glTF's factor of 1,
/// not multiplied by the default of 0, which would make it a mirror.
#[test]
fn an_unset_roughness_uses_the_map() {
    let render = |roughness: Option<f32>, map: Option<u8>| {
        let material = LocalState::new(PbrMaterial::new().color(Color::new_rgba(120, 120, 120, 255)).roughness(roughness));
        let textures = lens!(material.aomr_textures);
        let scene = Scene3d::new((Object::new(Mesh::plane(4.0, 4.0), material.clone()), DirectionalLight::new(Vector3::new(0.0, -1.0, 0.0)).intensity(3.0)))
            .ambient(WHITE, 0.0)
            .camera(Camera::perspective().look_at(Point3::new(0.0, 3.0, 0.001), Point3::new(0.0, 0.0, 0.0)));

        let mut harness = Harness::new(scene, Dimension::new(60.0, 60.0));
        if let Some(byte) = map {
            let id = store_flat_image(&mut harness, [byte, byte, byte, 255]);
            *textures.clone().value_mut() = AoMRTextures::BWSplit { ao_texture: None, m_texture: None, r_texture: Some(id.into()) };
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
    let sphere_material = LocalState::new(PbrMaterial::new()
        .color(Color::new_rgba(40, 90, 200, 255))
        .tiling(Vector2::new(3.0, 1.0)));
    let roughness = lens!(sphere_material.aomr_textures);
    let plane_material = LocalState::new(PbrMaterial::new().color(Color::new_rgba(200, 200, 200, 255)));
    let occlusion = lens!(plane_material.aomr_textures);

    let sphere = Scene3d::new((
        Object::new(Mesh::icosphere(1.0, 4), sphere_material.clone()),
        DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0)).intensity(4.0),
    ))
        .ambient(WHITE, 0.2)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let plane = Scene3d::new(Object::new(Mesh::plane(2.0, 2.0), plane_material.clone()))
        .ambient(WHITE, 1.0)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.2, 0.8), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((sphere, plane)).spacing(10.0), Dimension::new(410.0, 200.0));

    let roughness_stripes = store_stripes(&mut harness, 77, 230);
    *roughness.clone().value_mut() = AoMRTextures::BWSplit { ao_texture: None, m_texture: None, r_texture: Some(roughness_stripes.into()) };

    let occlusion_stripes = store_stripes(&mut harness, 64, 255);
    *occlusion.clone().value_mut() = AoMRTextures::BWSplit { ao_texture: Some(occlusion_stripes.into()), m_texture: None, r_texture: None };

    harness.snapshot("stripes");
}

/// The brightness of the middle of a plane with `material`, seen from straight above, after
/// `setup` has stored its images and set its states.
fn material_brightness(lighting: Lighting, material: impl IntoReadState<Material>, setup: impl FnOnce(&mut Harness)) -> u32 {
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
    let base = |strength: f32, roughness: f32| {
        PbrMaterial::new().color(Color::new_rgba(120, 120, 120, 255)).roughness(0.1).clearcoat(strength, roughness)
    };

    let plain = |byte: u8| {
        let (strength, roughness) = factors(byte as f32 / 255.0);
        material_brightness(Lighting::Highlight, base(strength, roughness), |_| {})
    };
    let mapped = |byte: u8| {
        let (strength, roughness) = factors(1.0);
        let material = LocalState::new(base(strength, roughness));
        material_brightness(Lighting::Highlight, material.clone(), |harness| {
            lens!(material.clearcoat_textures).set_value(maps(harness, byte));
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
    let image = |harness: &mut Harness, rgb: [u8; 3]| Some(store_flat_image(harness, [rgb[0], rgb[1], rgb[2], 255]).into());
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
        let material = LocalState::new(PbrMaterial::new().color(black).emissive(emissive));
        material_brightness(Lighting::Dark, material.clone(), |harness| {
            lens!(material.emissive_texture).set_value(Some(store_flat_image(harness, [byte, byte, byte, 255]).into()));
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
    let glowing_material = LocalState::new(PbrMaterial::new().color(Color::new_rgba(30, 30, 30, 255)).roughness(0.8).emissive(WHITE));
    let emission = lens!(glowing_material.emissive_texture);
    let coated_material = LocalState::new(PbrMaterial::new()
        .color(Color::new_rgba(40, 90, 200, 255))
        .roughness(0.2)
        .clearcoat(1.0, 0.8)
        .tiling(Vector2::new(3.0, 1.0)));
    let coat = lens!(coated_material.clearcoat_textures);

    let glowing = Scene3d::new(Object::new(Mesh::icosphere(1.0, 4), glowing_material.clone()))
        .ambient(WHITE, 0.1)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let coated = Scene3d::new((
        Object::new(Mesh::icosphere(1.0, 4), coated_material.clone()),
        DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0)).intensity(4.0),
    ))
        .ambient(WHITE, 0.2)
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.3, 3.2), Point3::new(0.0, 0.0, 0.0)))
        .frame(200.0, 200.0);

    let mut harness = Harness::new(HStack::new((glowing, coated)).spacing(10.0), Dimension::new(410.0, 200.0));

    let checker_id = ImageId::temp(ImageFormat::Png);
    store_checker(&mut harness, &checker_id);
    *emission.clone().value_mut() = Some(checker_id.into());

    let stripes = store_stripes(&mut harness, 0, 255);
    *coat.clone().value_mut() = ClearcoatTextures::BWSplit { clearcoat_texture: Some(stripes.into()), clearcoat_roughness_texture: None };

    harness.snapshot("glow_and_coat");
}

/// Tiling and sampling are states the material is made from: changing them redraws the material,
/// and changing them back gives the same picture again.
#[test]
fn tiling_and_sampling_follow_their_states() {
    let checker_id = ImageId::temp(ImageFormat::Png);
    let tiling = LocalState::new(Vector2::new(1.0f32, 1.0));
    let sampling = LocalState::new(SampleType::Linear);

    let material = {
        let checker_id = checker_id.clone();
        Map2::read_map(tiling.clone(), sampling.clone(), move |tiling: &Vector2<f32>, sampling: &SampleType| {
            PbrMaterial::new().color(checker_id.clone()).tiling(*tiling).sample_type(*sampling)
        })
    };
    let scene = Scene3d::new((Object::new(Mesh::plane(20.0, 20.0), material), DirectionalLight::new(Vector3::new(0.3, -1.0, -0.4)).intensity(4.0)))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.6, 3.0), Point3::new(0.0, 0.0, -2.0)));

    let mut harness = Harness::new(scene, Dimension::new(200.0, 150.0));
    store_checker(&mut harness, &checker_id);
    harness.render();
    let once = harness.pixels().as_raw().to_vec();

    let changed = |a: &[u8], b: &[u8]| a.iter().zip(b).filter(|(x, y)| x != y).count();

    *tiling.clone().value_mut() = Vector2::new(16.0, 16.0);
    harness.render();
    let tiled = harness.pixels().as_raw().to_vec();
    assert!(changed(&once, &tiled) > 10_000, "tiling 16 times changes only {} bytes", changed(&once, &tiled));

    *sampling.clone().value_mut() = SampleType::Nearest;
    harness.render();
    let nearest = harness.pixels().as_raw().to_vec();
    assert!(changed(&tiled, &nearest) > 1_000, "nearest sampling changes only {} bytes", changed(&tiled, &nearest));

    *sampling.clone().value_mut() = SampleType::Linear;
    *tiling.clone().value_mut() = Vector2::new(1.0, 1.0);
    harness.render();
    assert!(harness.pixels().as_raw() == &once[..], "back to one tile, linear, the picture differs");
}

/// One image shared by 2D and 3D, and a missing one, on a light background. Left:
/// `images/landscape.png` in an `Image`, scaled to fit. Middle: a cube with the same image as its
/// colour, from the same stored texture. Right: a cube whose image doesn't exist, drawn white with
/// the fallback texture, without a panic.
#[test]
fn an_image_shared_with_2d_and_a_missing_one() {
    let landscape = ImageId::new("images/landscape.png");

    let cube = |image: ImageId| {
        Scene3d::new((
            Object::new(Mesh::cube(1.0), PbrMaterial::new().color(image).roughness(Some(0.6f32)))
                .rotation(Euler::new(Deg(0.0), Deg(-30.0), Deg(0.0))),
            DirectionalLight::new(Vector3::new(1.0, -2.0, -1.5)).intensity(4.0),
        ))
            .camera(Camera::perspective().look_at(Point3::new(-1.2, 1.2, 2.0), Point3::new(0.0, 0.0, 0.0)))
            .frame(150.0, 150.0)
    };

    let widget = HStack::new((
        Image::new(landscape.clone()).resizeable().scaled_to_fit().frame(150.0, 150.0),
        cube(landscape.clone()),
        cube(ImageId::new("images/does_not_exist.png")),
    )).spacing(10.0).background(Rectangle::new().fill(Color::new_rgba(230, 230, 230, 255)));

    let mut harness = Harness::new(widget, Dimension::new(490.0, 170.0));
    harness.snapshot("shared_and_missing");

    let textures = harness.environment().get::<WgpuTextureStore>().unwrap();
    assert_eq!(textures.len(), 1, "the shared image is stored once, and the missing one not at all");
}

/// The checker on an unlit plane seen from straight above, so colours are exact: the plane's
/// material is `material` of the checker's id. Returns the picture's bytes.
fn checker_plane(mesh: Mesh, material: impl FnOnce(ImageId) -> PbrMaterial) -> Vec<u8> {
    let checker_id = ImageId::temp(ImageFormat::Png);
    let scene = Scene3d::new(Object::new(mesh, material(checker_id.clone()).unlit()))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 2.5, 0.001), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(120.0, 120.0));
    store_checker(&mut harness, &checker_id);
    harness.render();
    harness.pixels().as_raw().to_vec()
}

/// A map's own transform tiles it as the material's does, and the material's applies after it:
/// a map tiled 4 times renders exactly as the material tiled 4 times, and a map tiled twice in a
/// material tiled twice as a map tiled 4 times.
#[test]
fn a_maps_own_transform() {
    let plane = || Mesh::plane(2.0, 2.0);
    let tiled = |n: f32| Vector2::new(n, n);

    let material_tiled = checker_plane(plane(), |id| PbrMaterial::new().color(id).tiling(tiled(4.0)));
    let map_tiled = checker_plane(plane(), |id| PbrMaterial::new().color(MaterialTexture::new(id).tiling(tiled(4.0))));
    let both = checker_plane(plane(), |id| PbrMaterial::new().color(MaterialTexture::new(id).tiling(tiled(2.0))).tiling(tiled(2.0)));
    let once = checker_plane(plane(), |id| PbrMaterial::new().color(id));

    assert!(material_tiled != once, "tiling changes nothing");
    assert!(map_tiled == material_tiled, "the map tiled 4 times differs from the material tiled 4 times");
    assert!(both == material_tiled, "the map tiled twice in a material tiled twice differs from tiling 4 times");
}

/// The plane with its second texture coordinates set to `second` of its first.
fn with_second_coordinates(mesh: Mesh, second: impl Fn(Vector2<f32>) -> Vector2<f32>) -> Mesh {
    let mut vertices = mesh.vertices().to_vec();
    for vertex in &mut vertices {
        vertex.texture_coords_1 = second(vertex.texture_coords_0);
    }
    Mesh::new(vertices, mesh.indices().to_vec())
}

/// The bytes of `a` and `b` that differ by more than 2, as a share of all bytes.
fn share_different(a: &[u8], b: &[u8]) -> f32 {
    a.iter().zip(b).filter(|(x, y)| x.abs_diff(**y) > 2).count() as f32 / a.len() as f32
}

/// A map on the second texture coordinates reads them, and the other maps keep the first: on a
/// plane whose second coordinates are its first scaled 4 times, a map on set 1 renders as a map
/// tiled 4 times on set 0, and a map on set 0 as the untiled checker.
#[test]
fn a_map_on_the_second_texture_coordinates() {
    let plane = || with_second_coordinates(Mesh::plane(2.0, 2.0), |uv| uv * 4.0);

    let second = checker_plane(plane(), |id| PbrMaterial::new().color(MaterialTexture::new(id).uv_set(1)));
    let tiled = checker_plane(plane(), |id| PbrMaterial::new().color(MaterialTexture::new(id).tiling(Vector2::new(4.0, 4.0))));
    let first = checker_plane(plane(), |id| PbrMaterial::new().color(MaterialTexture::new(id)));
    let plain = checker_plane(Mesh::plane(2.0, 2.0), |id| PbrMaterial::new().color(id));

    assert!(share_different(&second, &first) > 0.1, "set 1 renders as set 0");
    assert!(share_different(&second, &tiled) < 0.005, "set 1 differs from set 0 tiled 4 times: {}", share_different(&second, &tiled));
    assert!(first == plain, "set 0 differs from the plain checker");
}

/// `images/landscape.png` on three planes, seen from above. Left: as it is. Middle: its own
/// transform scales it by 2, turns it 90° and offsets it by half, as `KHR_texture_transform` does:
/// turned copies, shifted by half a copy, each image turned a quarter clockwise (the sky on the
/// right), as Khronos's sample viewer turns TextureTransformTest's arrows. Right: on the second texture coordinates, which mirror the first left to
/// right, while its occlusion map, on the first set, darkens the plane's far half. Lit by ambient
/// light only, which occlusion darkens.
#[test]
fn map_transforms_and_coordinate_sets() {
    let landscape = ImageId::new("images/landscape.png");
    let half = LocalState::new(None::<ImageId>);

    fn view(material: impl IntoReadState<Material>, mesh: Mesh) -> impl Widget {
        // Ambient light only, which the occlusion map darkens.
        Scene3d::new(Object::new(mesh, material))
            .ambient(WHITE, 3.0)
            .camera(Camera::perspective().look_at(Point3::new(0.0, 2.6, 0.001), Point3::new(0.0, 0.0, 0.0)))
            .frame(150.0, 150.0)
    }
    let plain = view(PbrMaterial::new().color(landscape.clone()).roughness(Some(1.0f32)), Mesh::plane(2.0, 2.0));
    let turned = view(
        PbrMaterial::new()
            .color(MaterialTexture::new(landscape.clone()).offset_rotation_scale(Vector2::new(0.5, 0.0), Rad(std::f32::consts::FRAC_PI_2), Vector2::new(2.0, 2.0)))
            .roughness(Some(1.0f32)),
        Mesh::plane(2.0, 2.0),
    );
    let mirrored_landscape = landscape.clone();
    let mirrored = view(
        Map1::read_map(half.clone(), move |id: &Option<ImageId>| {
            let occlusion = match id {
                Some(id) => AoMRTextures::BWSplit { ao_texture: Some(id.into()), m_texture: None, r_texture: None },
                None => AoMRTextures::None,
            };
            PbrMaterial::new()
                .color(MaterialTexture::new(mirrored_landscape.clone()).uv_set(1))
                .aomr_textures(occlusion)
                .roughness(1.0)
        }),
        with_second_coordinates(Mesh::plane(2.0, 2.0), |uv| Vector2::new(1.0 - uv.x, uv.y)),
    );

    let mut harness = Harness::new(HStack::new((plain, turned, mirrored)).spacing(10.0), Dimension::new(470.0, 150.0));
    // An occlusion map dark in its top half, the plane's far half.
    let mut data = Vec::with_capacity(16 * 16 * 4);
    for row in 0..16 {
        let value = if row < 8 { 60 } else { 255 };
        for _ in 0..16 {
            data.extend_from_slice(&[value, value, value, 255]);
        }
    }
    let id = ImageId::temp(ImageFormat::Png);
    WgpuImageContext.update_texture(&id, Texture { width: 16, height: 16, bytes_per_row: 64, format: TextureFormat::RGBA8, data: &data }, harness.environment_mut());
    *half.clone().value_mut() = Some(id);
    harness.snapshot("map_transforms");
}

/// Store a 4 × 4 image under `id`: its left half `left`, its right half `right`.
fn store_halves(harness: &mut Harness, id: &ImageId, left: [u8; 4], right: [u8; 4]) {
    let data: Vec<u8> = (0..16).flat_map(|i| if i % 4 < 2 { left } else { right }).collect();
    let texture = Texture { width: 4, height: 4, bytes_per_row: 16, format: TextureFormat::RGBA8, data: &data };
    WgpuImageContext.update_texture(id, texture, harness.environment_mut());
}

/// Three strips seen from above, each with a red and blue image whose coordinates run from −1 to
/// 2 across it, so it covers the image three times over: repeated, clamped and mirrored. Six
/// points along each strip, the middles of its sixths, tell them apart whichever way u runs on
/// screen: repeated, the colour changes at every point; clamped, each end is one colour; mirrored,
/// it changes at every other point (A BB AA B). The maps are nearest-filtered though the
/// material's default is linear, so each point is exactly red or blue: a map's own filtering wins.
#[test]
fn maps_repeat_clamp_or_mirror() {
    use carbide_3d::material::material_texture::MaterialTexture;
    use carbide_3d::material::sample_type::SampleType;
    use carbide_3d::material::texture_sampler::{Filtering, Wrap};
    use carbide_core::math::{Rad, Vector2};

    let id = ImageId::temp(ImageFormat::Png);
    let strip = |wrap: Wrap, z: f32| {
        let map = MaterialTexture::new(id.clone())
            .offset_rotation_scale(Vector2::new(-1.0, 0.0), Rad(0.0), Vector2::new(3.0, 1.0))
            .wrap(wrap)
            .filtering(Filtering::NEAREST);
        let material = PbrMaterial::new().color(map).sample_type(SampleType::Linear).unlit();
        Object::new(Mesh::plane(3.0, 0.8), material).position(Vector3::new(0.0, 0.0, z))
    };
    let scene = Scene3d::new((strip(Wrap::Repeat, -1.1), strip(Wrap::ClampToEdge, 0.0), strip(Wrap::MirroredRepeat, 1.1)))
        .camera(Camera::orthographic(3.6).look_at(Point3::new(0.0, 5.0, 0.001), Point3::new(0.0, 0.0, 0.0)));

    let mut harness = Harness::new(scene, Dimension::new(300.0, 300.0));
    store_halves(&mut harness, &id, [255, 0, 0, 255], [0, 0, 255, 255]);
    harness.render();

    // 300 pixels for 3.6 metres: the strips span x from 25 to 275, and lie at y 8.3 metres apart.
    let picture = harness.pixels();
    let row = |z: f32| -> Vec<bool> {
        let y = (150.0 + z * 300.0 / 3.6).round() as u32;
        (0..6).map(|i| {
            let [r, g, b, _] = picture.get_pixel((25.0 + (i as f32 + 0.5) * 250.0 / 6.0).round() as u32, y).0;
            assert!(g < 20 && (r > 235 && b < 20 || b > 235 && r < 20), "not pure red or blue at {i}, z {z}: {:?}", [r, g, b]);
            r > 128
        }).collect()
    };
    let changes = |points: &[bool]| points.windows(2).map(|pair| pair[0] != pair[1]).collect::<Vec<_>>();

    assert_eq!(changes(&row(-1.1)), [true; 5], "repeated: {:?}", row(-1.1));
    let clamped = row(0.0);
    assert!(clamped[0] == clamped[1] && clamped[4] == clamped[5] && clamped[0] != clamped[5], "clamped: {clamped:?}");
    assert_eq!(changes(&row(1.1)), [true, false, true, false, true], "mirrored: {:?}", row(1.1));
    harness.snapshot("strips");
}
