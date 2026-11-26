//! glTF import: Khronos's sample models (`ressources/gltf`, each with its licence), loaded,
//! checked against what their files say, and drawn as snapshots.
//!
//! Every snapshot frames its model from the model's bounds, seen from a direction, lit by a
//! directional light from the upper left front and some ambient light, unless it says otherwise.
//! Khronos's screenshots of the models (in their repository's `screenshot` folders) are rendered
//! with image-based lighting, which carbide doesn't have yet, so metals look darker here.

use std::path::PathBuf;

use carbide_3d::camera::Camera;
use carbide_3d::gltf::{load_gltf, GltfLight, GltfModel, GltfScene};
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::albedo_component::AlbedoComponent;
use carbide_3d::material::ao_mr_textures::AoMRTextures;
use carbide_3d::material::clearcoat_textures::ClearcoatTextures;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::material::transparency::Transparency;
use carbide_3d::{Mesh, Node3dExt, Object, Scene3d};
use carbide_core::color::{ColorExt, WHITE};
use carbide_core::draw::{Dimension, ImageId};
use carbide_core::math::{EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, SquareMatrix, Vector3, Vector4};
use carbide_core::state::ReadState;
use carbide_testing::Harness;

mod common;

/// The sample model `name`, as a `.glb`, or as a `.gltf` for the two models that only come as one.
fn model(name: &str) -> PathBuf {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ressources/gltf").join(name);
    let glb = folder.join(format!("{name}.glb"));
    if glb.exists() { glb } else { folder.join(format!("{name}.gltf")) }
}

fn scene(name: &str) -> GltfScene {
    load_gltf(model(name)).unwrap_or_else(|error| panic!("{name}: {error}"))
}

/// The transform that centres `scene` on the origin, and scales it to 2 metres across at its
/// widest if it is under 10 centimetres or over 100 metres: MetalRoughSpheresNoTextures is under a
/// centimetre across, inside the camera's near plane at its own size. Models of sensible sizes
/// keep theirs, since scaling a model scales the distances its own lights fall off over.
fn normalising(scene: &GltfScene) -> Matrix4<f32> {
    let (min, max) = scene.bounds().expect("the model has no vertices");
    let extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
    let centre = Vector3::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0, (min.z + max.z) / 2.0);
    let scale = if (0.1..=100.0).contains(&extent) { 1.0 } else { 2.0 / extent };
    Matrix4::from_scale(scale) * Matrix4::from_translation(-centre)
}

/// A camera looking at the origin from `direction`, as near as fits all of `scene`, normalised
/// (see [`normalising`]), in a view of `aspect` (width over height), with a little room around it.
fn framing(scene: &GltfScene, direction: Vector3<f32>, aspect: f32) -> Camera {
    let (min, max) = scene.bounds().expect("the model has no vertices");
    let normalise = normalising(scene);

    let backwards = direction.normalize();
    let right = Vector3::unit_y().cross(backwards).normalize();
    let up = backwards.cross(right);
    let tan = 30f32.to_radians().tan();

    // Each corner fits when the camera is at least this far from the centre.
    let mut distance = 0.0f32;
    for corner in 0..8 {
        let p = normalise * Vector4::new(
            if corner & 1 == 0 { min.x } else { max.x },
            if corner & 2 == 0 { min.y } else { max.y },
            if corner & 4 == 0 { min.z } else { max.z },
            1.0,
        );
        let p = p.truncate();
        let towards_camera = p.dot(backwards);
        distance = distance
            .max(p.dot(right).abs() / (tan * aspect) + towards_camera)
            .max(p.dot(up).abs() / tan + towards_camera);
    }

    let eye = Point3::from_vec(backwards * distance * 1.1);
    Camera::perspective().look_at(eye, Point3::new(0.0, 0.0, 0.0))
}

/// The model of `scene`, normalised (see [`normalising`]), with or without its own lights.
fn normalised_model(scene: GltfScene, lights: bool) -> impl carbide_3d::Node3d {
    let transform = normalising(&scene);
    GltfModel::new(scene).lights(lights).transform(transform)
}

/// The usual light: from the upper left front.
fn light() -> DirectionalLight<Vector3<f32>, carbide_core::draw::Color, f32> {
    DirectionalLight::new(Vector3::new(0.5, -0.8, -0.6)).intensity(3.0f32)
}

/// A snapshot of the sample model `name`, seen from `direction` in a view of `size`, lit by the
/// usual light and ambient light, without the file's own lights.
fn snapshot(name: &str, snapshot_name: &str, direction: Vector3<f32>, size: (f64, f64)) {
    let loaded = scene(name);
    let camera = framing(&loaded, direction, (size.0 / size.1) as f32);
    let view = Scene3d::new((normalised_model(loaded, false), light()))
        .ambient(WHITE, 0.6)
        .camera(camera);

    let mut harness = Harness::new(view, Dimension::new(size.0, size.1));
    harness.snapshot(snapshot_name);
}

const FRONT: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
/// From the front, a little to the right and above, to show depth.
const FRONT_RIGHT_ABOVE: Vector3<f32> = Vector3::new(0.5, 0.4, 1.0);

// --- Loading: what the files say becomes the right nodes and materials ---

/// A file that doesn't exist is an error, not a panic.
#[test]
fn a_missing_file_is_an_error() {
    assert!(load_gltf(model("NoSuchModel")).is_err());
}

/// BoxTextured: one node turned to stand the box up, one mesh with normals and texture
/// coordinates, and a material whose base colour is an image inside the `.glb`, with its
/// metallic factor 0 and its roughness the file's.
#[test]
fn box_textured_loads_its_mesh_and_embedded_texture() {
    let loaded = scene("BoxTextured");
    let primitives = loaded.primitives();
    assert_eq!(primitives.len(), 1);

    let (mesh, material, _) = primitives[0];
    assert_eq!(mesh.indices().len(), 36);
    assert!(mesh.vertices().iter().all(|vertex| (vertex.normal.magnitude() - 1.0).abs() < 1e-3));

    let texture = material.albedo.value().to_texture().cloned().expect("no base colour texture");
    assert!(matches!(texture.image, ImageId::InMemory(..)), "the embedded image has a file id: {:?}", texture.image);
    assert!(material.textures().contains(&texture.image));
    assert_eq!(*material.metallic.value(), 0.0);
    assert!(loaded.unsupported.is_empty(), "{:?}", loaded.unsupported);
}

/// AlphaBlendModeTest: opaque, masked (with the default cutoff of 0.5 and the file's own) and
/// blended materials.
#[test]
fn alpha_modes_become_transparency() {
    let loaded = scene("AlphaBlendModeTest");
    let modes: Vec<Transparency> = loaded.primitives().iter().map(|(_, material, _)| material.transparency).collect();

    assert!(modes.contains(&Transparency::Opaque));
    assert!(modes.contains(&Transparency::Blend));
    assert!(modes.contains(&Transparency::Cutout { cutout: 0.5 }), "no default cutoff: {modes:?}");
    assert!(modes.iter().any(|mode| matches!(mode, Transparency::Cutout { cutout } if *cutout != 0.5)), "no cutoff of the file's own: {modes:?}");
}

/// TextureTransformTest: maps with their own `KHR_texture_transform`, which become each map's
/// transform: some offset, some turned, some scaled.
#[test]
fn texture_transforms_are_per_map() {
    let loaded = scene("TextureTransformTest");
    let transforms: Vec<Matrix3<f32>> = loaded.primitives().iter()
        .filter_map(|(_, material, _)| material.albedo.value().to_texture().map(|texture| texture.transform))
        .collect();

    assert!(transforms.iter().any(|t| *t == Matrix3::identity()), "no untransformed map");
    // A translation shows in the third column; a turn makes the first column leave the u axis;
    // a scale changes its length.
    assert!(transforms.iter().any(|t| t.z.x != 0.0 || t.z.y != 0.0), "no offset");
    assert!(transforms.iter().any(|t| t.x.y.abs() > 1e-3), "no rotation");
    assert!(transforms.iter().any(|t| (t.x.magnitude() - 1.0).abs() > 1e-3), "no scale");
}

/// MultiUVTest: a map on the second texture coordinates.
#[test]
fn maps_keep_their_texture_coordinate_set() {
    let loaded = scene("MultiUVTest");
    let sets: Vec<u8> = loaded.primitives().iter().flat_map(|(_, material, _)| {
        let albedo = material.albedo.value().to_texture().map(|texture| texture.uv_set);
        let emissive = material.emissive_texture.value().as_ref().map(|texture| texture.uv_set);
        [albedo, emissive]
    }).flatten().collect();

    assert!(sets.contains(&0) && sets.contains(&1), "texture coordinate sets: {sets:?}");
    assert!(loaded.primitives().iter().any(|(mesh, ..)| mesh.vertices().iter().any(|vertex| vertex.texture_coords_1.magnitude() > 0.0)), "no second coordinates");
}

/// ClearCoatTest: clear-coat factors and maps (`KHR_materials_clearcoat`), read from the file's
/// JSON. Its clear-coat normal maps are reported as unsupported.
#[test]
fn clear_coat_comes_from_its_extension() {
    let loaded = scene("ClearCoatTest");
    let coats: Vec<f32> = loaded.primitives().iter().map(|(_, material, _)| *material.clearcoat.value()).collect();

    assert!(coats.iter().any(|coat| *coat == 1.0), "no full coat: {coats:?}");
    assert!(coats.iter().any(|coat| *coat == 0.0), "no uncoated material: {coats:?}");
    assert!(loaded.primitives().iter().any(|(_, material, _)| matches!(*material.clearcoat_textures.value(), ClearcoatTextures::GltfSplit { .. })), "no clear-coat maps");
    assert!(loaded.unsupported.iter().any(|what| what == "clear-coat normal maps"), "{:?}", loaded.unsupported);
}

/// EmissiveStrengthTest: emission strengths above 1 (`KHR_materials_emissive_strength`).
#[test]
fn emission_strength_comes_from_its_extension() {
    let loaded = scene("EmissiveStrengthTest");
    let strengths: Vec<f32> = loaded.primitives().iter().map(|(_, material, _)| *material.emissive_strength.value()).collect();
    assert!(strengths.iter().any(|strength| *strength > 1.0), "strengths: {strengths:?}");
}

/// UnlitTest: unlit materials (`KHR_materials_unlit`).
#[test]
fn unlit_materials_are_unlit() {
    let loaded = scene("UnlitTest");
    assert!(loaded.primitives().iter().all(|(_, material, _)| material.unlit));
}

/// TwoSidedPlane: a double-sided material.
#[test]
fn double_sided_materials_are_double_sided() {
    let loaded = scene("TwoSidedPlane");
    assert!(loaded.primitives().iter().all(|(_, material, _)| *material.double_sided.value()));
}

/// The occlusion, roughness and metallic maps: DamagedHelmet keeps occlusion in its own image,
/// with roughness and metallic in green and blue of another (swizzled split).
#[test]
fn occlusion_roughness_and_metallic_maps() {
    let loaded = scene("DamagedHelmet");
    let (_, material, _) = loaded.primitives()[0];
    match &*material.aomr_textures.value() {
        AoMRTextures::SwizzledSplit { ao_texture: Some(ao), mr_texture: Some(mr) } => assert_ne!(ao.image, mr.image),
        AoMRTextures::Combined { texture: Some(_) } => {}
        other => panic!("DamagedHelmet's maps: {other:?}"),
    }
    assert!(material.normal.value().to_texture().is_some());
    assert!(material.emissive_texture.value().is_some());
}

/// VertexColorTest and BoxVertexColors: vertex colours multiply the base colour, read as linear.
#[test]
fn vertex_colours_are_linear() {
    let loaded = scene("BoxVertexColors");
    let (_, material, _) = loaded.primitives()[0];
    assert!(matches!(*material.albedo.value(), AlbedoComponent::ValueVertex { srgb: false, .. } | AlbedoComponent::TextureVertexValue { srgb: false, .. }), "{:?}", material.albedo.value());
}

/// NegativeScaleTest: nodes mirrored by a negative scale.
#[test]
fn negative_scales_are_kept() {
    let loaded = scene("NegativeScaleTest");
    assert!(loaded.primitives().iter().any(|(_, _, transform)| transform.determinant() < 0.0));
}

/// LightsPunctualLamp: its five `KHR_lights_punctual` point lights, in candela as the file gives
/// them, with their colours converted from linear to sRGB (the warm bulb's green, 0.63 in the
/// file, is 0.82).
#[test]
fn lights_come_from_their_extension() {
    let loaded = scene("LightsPunctualLamp");
    let lights = loaded.lights();
    assert_eq!(lights.len(), 5, "{lights:?}");

    let intensities: Vec<f32> = lights.iter().map(|(light, _)| match light {
        GltfLight::Point { intensity, range: None, .. } => *intensity,
        other => panic!("not a point light without range: {other:?}"),
    }).collect();
    assert_eq!(intensities, vec![15.0, 1.5, 80.0, 80.0, 180.0]);

    let GltfLight::Point { color, .. } = lights[0].0 else { unreachable!() };
    assert!((color.green() - 0.816).abs() < 0.01, "{color:?}");
}

/// Which extensions and features each file uses that carbide leaves out, reported once.
#[test]
fn unsupported_features_are_reported() {
    // The texture settings test clamps and mirrors some textures.
    let settings = scene("TextureSettingsTest");
    assert!(settings.unsupported.iter().any(|what| what == "texture wrap modes other than repeat"), "{:?}", settings.unsupported);
    // Reported once each.
    let mut deduplicated = settings.unsupported.clone();
    deduplicated.dedup();
    assert_eq!(deduplicated, settings.unsupported);
}

// --- Mirroring, in the renderer ---

/// A cube mirrored by a negative scale is the same cube, so it draws the same: its triangles turn
/// clockwise on screen, and the renderer takes that as their front, as glTF requires. Drawn as
/// before, the mirrored cube would show its inside.
#[test]
fn a_mirrored_cube_draws_as_the_cube() {
    let picture = |scale: Vector3<f32>| {
        let cube = Object::new(Mesh::cube(1.0), PbrMaterial::new().color(WHITE).roughness(Some(0.6f32))).scale(scale);
        let scene = Scene3d::new((cube, light()))
            .camera(Camera::perspective().look_at(Point3::new(1.5, 1.2, 2.5), Point3::new(0.0, 0.0, 0.0)));
        let mut harness = Harness::new(scene, Dimension::new(120.0, 120.0));
        harness.render();
        harness.pixels().as_raw().to_vec()
    };

    let plain = picture(Vector3::new(1.0, 1.0, 1.0));
    let mirrored = picture(Vector3::new(-1.0, 1.0, 1.0));
    let different = plain.iter().zip(&mirrored).filter(|(a, b)| a.abs_diff(**b) > 2).count();
    assert!(different < plain.len() / 200, "{different} bytes differ");
}

// --- Snapshots ---

#[test]
fn box_textured() {
    snapshot("BoxTextured", "box_textured", FRONT_RIGHT_ABOVE, (300.0, 300.0));
}

#[test]
fn box_vertex_colors() {
    snapshot("BoxVertexColors", "box_vertex_colors", FRONT_RIGHT_ABOVE, (300.0, 300.0));
}

#[test]
fn vertex_color_test() {
    snapshot("VertexColorTest", "vertex_color_test", FRONT, (400.0, 300.0));
}

/// Node rotations, as quaternions and as matrices: each arrow must point at the target of its own
/// colour.
#[test]
fn orientation_test() {
    // Red, green and blue: quaternions, on the front, right and top.
    snapshot("OrientationTest", "front", FRONT_RIGHT_ABOVE, (400.0, 400.0));
    // Cyan, magenta and yellow: matrices, on the back, left and bottom.
    snapshot("OrientationTest", "back", Vector3::new(-0.5, -0.4, -1.0), (400.0, 400.0));
}

/// Texture coordinates: the numbers must read the right way round.
#[test]
fn texture_coordinate_test() {
    snapshot("TextureCoordinateTest", "texture_coordinate_test", FRONT, (400.0, 400.0));
}

/// `KHR_texture_transform`: every square should read "Correct" where it is supported.
#[test]
fn texture_transform_test() {
    snapshot("TextureTransformTest", "texture_transform_test", FRONT, (450.0, 300.0));
}

#[test]
fn texture_transform_multi_test() {
    snapshot("TextureTransformMultiTest", "texture_transform_multi_test", FRONT, (500.0, 400.0));
}

/// Wrap modes (clamped and mirrored, unsupported: they repeat) and nearest sampling.
#[test]
fn texture_settings_test() {
    snapshot("TextureSettingsTest", "texture_settings_test", FRONT, (450.0, 450.0));
}

/// Maps on the second texture coordinates.
#[test]
fn multi_uv_test() {
    snapshot("MultiUVTest", "multi_uv_test", FRONT_RIGHT_ABOVE, (300.0, 300.0));
}

/// Normal maps with tangents from the file: the light should catch each hemisphere's bumps on
/// the same side.
#[test]
fn normal_tangent_test() {
    snapshot("NormalTangentTest", "normal_tangent_test", FRONT, (640.0, 480.0));
}

/// Normal maps on mirrored texture coordinates: each half's bumps lit on the same side.
#[test]
fn normal_tangent_mirror_test() {
    snapshot("NormalTangentMirrorTest", "normal_tangent_mirror_test", FRONT, (640.0, 480.0));
}

/// Opaque, masked and blended materials. Blending isn't sorted yet (Materials, item 9).
#[test]
fn alpha_blend_mode_test() {
    snapshot("AlphaBlendModeTest", "alpha_blend_mode_test", FRONT, (600.0, 300.0));
}

/// Metallic from 0 to 1 up the grid and roughness from 0 to 1 across, as the usual chart.
#[test]
fn metal_rough_spheres_no_textures() {
    snapshot("MetalRoughSpheresNoTextures", "metal_rough_spheres_no_textures", FRONT, (400.0, 400.0));
}

/// Clear-coat factors and maps. The coat only widens the highlight for now (A real clear-coat
/// layer).
#[test]
fn clear_coat_test() {
    snapshot("ClearCoatTest", "clear_coat_test", FRONT, (400.0, 500.0));
}

/// Emission strengths from 1 upwards: brighter cubes clip and desaturate in the tone mapping.
#[test]
fn emissive_strength_test() {
    snapshot("EmissiveStrengthTest", "emissive_strength_test", FRONT, (500.0, 250.0));
}

/// Mirrored nodes must look like their unmirrored neighbours, mirrored, and never inside out.
#[test]
fn negative_scale_test() {
    snapshot("NegativeScaleTest", "negative_scale_test", FRONT, (500.0, 350.0));
}

#[test]
fn unlit_test() {
    snapshot("UnlitTest", "unlit_test", FRONT, (400.0, 250.0));
}

/// A double-sided plane lying flat, from above and from below: both sides drawn, each lit as the
/// side facing the light would be (the underside, facing away from the light, by ambient light).
#[test]
fn two_sided_plane() {
    snapshot("TwoSidedPlane", "above", FRONT_RIGHT_ABOVE, (300.0, 300.0));
    snapshot("TwoSidedPlane", "below", Vector3::new(-0.5, -0.6, -1.0), (300.0, 300.0));
}

/// The damaged helmet: every map, all embedded JPEGs, from the front left, as Khronos's
/// screenshot, and from the side.
#[test]
fn damaged_helmet() {
    snapshot("DamagedHelmet", "front_left", Vector3::new(-0.8, 0.2, 1.0), (400.0, 400.0));
    snapshot("DamagedHelmet", "side", Vector3::new(1.0, 0.2, 0.3), (400.0, 400.0));
}

/// The lamp lit by its own `KHR_lights_punctual` lights, with a little ambient light; and with
/// them turned off, lit as the other models are.
#[test]
fn lights_punctual_lamp() {
    let loaded = scene("LightsPunctualLamp");
    let camera = framing(&loaded, FRONT_RIGHT_ABOVE, 1.0);

    let own_lights = Scene3d::new(normalised_model(loaded.clone(), true))
        .ambient(WHITE, 0.1)
        .camera(camera.clone());
    let mut harness = Harness::new(own_lights, Dimension::new(400.0, 400.0));
    harness.snapshot("own_lights");

    let usual = Scene3d::new((normalised_model(loaded, false), light()))
        .ambient(WHITE, 0.6)
        .camera(camera);
    let mut harness = Harness::new(usual, Dimension::new(400.0, 400.0));
    harness.snapshot("usual_lights");
}

