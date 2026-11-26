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
