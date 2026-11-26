//! OBJ models with their MTL materials.

use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::{Mesh, ObjModel, Scene3d};
use carbide_core::color::{Color, ColorExt, BLUE};
use carbide_core::draw::Dimension;
use carbide_core::math::{Point3, Vector3};
use carbide_core::state::ReadState;
use carbide_core::widget::WidgetExt;
use carbide_testing::Harness;

mod common;

const RENDER_BALL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/ressources/renderball.obj");

/// The render ball's meshes and their materials, from `renderball.mtl`. Pins down that each mesh
/// gets the material its `usemtl` names: the inner and outer balls have `Ks 0.5`, so a reflectance
/// of 0.25, and the stand has `Ks 1.0`, so 0.5. The inner ball's `Kd` is carbide's `BLUE` in
/// linear, so it loads as exactly `BLUE`; the others are grey.
#[test]
fn each_mesh_gets_its_material() {
    let parts = Mesh::load_obj_with_materials(RENDER_BALL).expect("the render ball to load");
    assert_eq!(parts.len(), 3, "a mesh per material");

    let mut reflectances = parts.iter().map(|(_, material)| *material.reflectance.value()).collect::<Vec<_>>();
    reflectances.sort_by(f32::total_cmp);
    assert_eq!(reflectances, vec![0.25, 0.25, 0.5]);

    let colors = parts.iter().map(|(_, material)| material.albedo.value().to_value()).collect::<Vec<_>>();
    let is_blue = |color: &Color| {
        (color.red() - BLUE.red()).abs() < 0.002 && (color.green() - BLUE.green()).abs() < 0.002 && (color.blue() - BLUE.blue()).abs() < 0.002
    };
    assert_eq!(colors.iter().filter(|color| is_blue(color)).count(), 1, "one blue ball: {colors:?}");

    for color in colors.iter().filter(|color| !is_blue(color)) {
        // Kd 0.64 and 0.7 are linear: about 0.82 and 0.85 in sRGB.
        assert!(color.red() > 0.8 && color.red() < 0.87, "{color:?}");
    }
}

/// The render ball as an `ObjModel`, with its file's materials. Pins down that the model renders
/// with them: a blue inner ball in a grey shell, with the stand's highlights stronger and wider.
#[test]
fn the_render_ball_with_its_materials() {
    let ball = ObjModel::load(RENDER_BALL).expect("the render ball to load");
    let light = DirectionalLight::new(Vector3::new(1.0, -2.0, -1.0)).intensity(5.0);

    let scene = Scene3d::new((ball, light))
        .camera(Camera::perspective().look_at(Point3::new(0.0, 1.2, 1.8), Point3::new(0.0, 0.5, 0.0)))
        .border().color(BLUE).padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("render_ball");
}

/// Write a quad facing +Z, two metres across, with texture coordinates in OBJ's convention ((0, 0)
/// at the bottom left), an unlit material (`illum 0`) using `map_Kd`, and a 32×32 image whose
/// quadrants are red (top left), green (top right), blue (bottom left) and white (bottom right).
/// `map_options` go before the image's name. Returns the OBJ file's path.
fn write_marker_quad(name: &str, map_options: &str) -> std::path::PathBuf {
    let folder = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&folder).unwrap();

    let image = image::RgbaImage::from_fn(32, 32, |x, y| match (x < 16, y < 16) {
        (true, true) => image::Rgba([255, 0, 0, 255]),
        (false, true) => image::Rgba([0, 255, 0, 255]),
        (true, false) => image::Rgba([0, 0, 255, 255]),
        (false, false) => image::Rgba([255, 255, 255, 255]),
    });
    image.save(folder.join("marker.png")).unwrap();

    std::fs::write(folder.join("quad.mtl"), format!("newmtl marker\nKd 1 1 1\nillum 0\nmap_Kd {map_options} marker.png\n")).unwrap();
    std::fs::write(folder.join("quad.obj"), "mtllib quad.mtl\n\
        v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\n\
        vt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\n\
        vn 0 0 1\n\
        usemtl marker\n\
        f 1/1/1 2/2/1 3/3/1 4/4/1\n").unwrap();

    folder.join("quad.obj")
}

/// The marker quad, seen straight on and filling the middle of a 200×200 view.
fn render_marker_quad(obj: &std::path::Path) -> Harness {
    let scene = Scene3d::new(ObjModel::load(obj).unwrap())
        .camera(Camera::perspective().look_at(Point3::new(0.0, 0.0, 3.0), Point3::new(0.0, 0.0, 0.0)));
    let mut harness = Harness::new(scene, Dimension::new(200.0, 200.0));
    harness.render();
    harness
}

/// The colour at (x, y), as the nearest of red, green, blue and white.
fn quadrant_colour(harness: &mut Harness, x: u32, y: u32) -> &'static str {
    let [r, g, b, _] = harness.pixels().get_pixel(x, y).0;
    match (r > 128, g > 128, b > 128) {
        (true, false, false) => "red",
        (false, true, false) => "green",
        (false, false, true) => "blue",
        (true, true, true) => "white",
        _ => "other",
    }
}

/// An OBJ file's `map_Kd` texture shows the right way up and the right way round: OBJ's texture
/// coordinates have v pointing up, and are flipped on loading. The image's top-left quadrant is
/// at the quad's top left, as it would be in any image viewer.
#[test]
fn an_obj_colour_map_reads_the_right_way_round() {
    let obj = write_marker_quad("marker_quad", "");
    let mut harness = render_marker_quad(&obj);

    // The quad spans roughly 60 to 140 in both directions; sample each quadrant's middle.
    assert_eq!(quadrant_colour(&mut harness, 80, 80), "red");
    assert_eq!(quadrant_colour(&mut harness, 120, 80), "green");
    assert_eq!(quadrant_colour(&mut harness, 80, 120), "blue");
    assert_eq!(quadrant_colour(&mut harness, 120, 120), "white");

    harness.snapshot("marker");
}

/// `-s` on `map_Kd` tiles the texture: at a scale of 2, each quarter of the quad shows the whole
/// image, so its top-left quarter has all four colours, still the right way round.
#[test]
fn an_obj_map_scale_tiles_the_texture() {
    let obj = write_marker_quad("marker_quad_tiled", "-s 2 2 1");
    let mut harness = render_marker_quad(&obj);

    // The quad's top-left quarter spans roughly 60 to 100; sample its own quadrants.
    assert_eq!(quadrant_colour(&mut harness, 70, 70), "red");
    assert_eq!(quadrant_colour(&mut harness, 90, 70), "green");
    assert_eq!(quadrant_colour(&mut harness, 70, 90), "blue");
    assert_eq!(quadrant_colour(&mut harness, 90, 90), "white");
}
