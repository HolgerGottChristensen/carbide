//! A playground for carbide's 3D: a scene with its options in a side bar.
//!
//! Every option is a state the scene reads, so it changes the picture as it is set: the model, one
//! or two views of it, the renderer (switched live), the material, the light and the camera. Drag
//! or scroll in a view to orbit its camera, and pinch to zoom.
//!
//! Run it with `cargo run -p playground_3d`.

mod meshes;
mod playground;

use carbide::draw::Dimension;
use carbide::{Application, Window};
use playground::{playground, Options};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "3D Playground - Carbide",
        Dimension::new(1200.0, 800.0),
        playground(Options::new()),
    ));

    application.launch();
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Position};
    use carbide::state::{ReadState, State};
    use carbide::testing::Harness;

    use super::playground::{playground, LightColor, LightKind, Model, Options, ProjectionKind, Renderer, Surface, Views};

    /// Every model, in one and two views, with both projections and both renderers, renders, and
    /// the main view can be dragged. A smoke test: it renders each combination without comparing
    /// pixels.
    #[test]
    fn every_option_renders() {
        let options = Options::new();
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();

        for model in [Model::Empty, Model::Cube, Model::Sphere, Model::Cubes, Model::RenderBall, Model::MaterialGrid] {
            for views in [Views::One, Views::Two] {
                for projection in [ProjectionKind::Perspective, ProjectionKind::Orthographic] {
                    for renderer in [Renderer::Wgpu, Renderer::None] {
                        *options.model.clone().value_mut() = model;
                        *options.views.clone().value_mut() = views;
                        *options.projection.clone().value_mut() = projection;
                        *options.renderer.clone().value_mut() = renderer;
                        harness.render();
                    }
                }
            }
        }

        *options.axes.clone().value_mut() = true;
        harness.render();

        harness.drag(Position::new(300.0, 400.0), Position::new(400.0, 400.0), 5);
        harness.render();

        // Each kind of light.
        for kind in [LightKind::Point, LightKind::Spot, LightKind::Directional] {
            *options.light_kind.clone().value_mut() = kind;
            harness.render();
        }

        // Hovering and clicking the model selects it; clicking again deselects it.
        *options.model.clone().value_mut() = Model::Cube;
        *options.projection.clone().value_mut() = ProjectionKind::Perspective;
        *options.views.clone().value_mut() = Views::One;
        harness.render();
        harness.click(Position::new(445.0, 400.0));
        harness.render();
        harness.click(Position::new(445.0, 400.0));
        harness.render();

        // Outlined, thick, then not.
        *options.outlined.clone().value_mut() = true;
        *options.outline_thickness.clone().value_mut() = 8.0;
        harness.render();
        *options.outlined.clone().value_mut() = false;
        harness.render();

        // The render ball with the side bar's materials, then its file's again.
        *options.model.clone().value_mut() = Model::RenderBall;
        *options.file_materials.clone().value_mut() = false;
        harness.render();
        *options.file_materials.clone().value_mut() = true;
        harness.render();

        // Every material value at once, then back.
        *options.metallic.clone().value_mut() = 1.0;
        *options.reflectance.clone().value_mut() = 1.0;
        *options.clearcoat.clone().value_mut() = 0.8;
        *options.ambient_occlusion.clone().value_mut() = 0.2;
        *options.emissive.clone().value_mut() = carbide::color::GREEN;
        *options.emissive_strength.clone().value_mut() = 2.0;
        *options.vertex_colors.clone().value_mut() = true;
        harness.render();
        *options.metallic.clone().value_mut() = 0.0;
        *options.emissive.clone().value_mut() = carbide::color::BLACK;
        *options.vertex_colors.clone().value_mut() = false;
        harness.render();

        // Each light colour, and a temperature.
        for color in [LightColor::Warm, LightColor::Blue, LightColor::Temperature, LightColor::White] {
            *options.light_color.clone().value_mut() = color;
            harness.render();
        }
        *options.light_color.clone().value_mut() = LightColor::Temperature;
        *options.light_temperature.clone().value_mut() = 2000.0;
        harness.render();
        *options.light_color.clone().value_mut() = LightColor::White;

        // Shadows on a floor, from the directional and the spot light.
        *options.floor.clone().value_mut() = true;
        *options.shadows.clone().value_mut() = true;
        for kind in [LightKind::Spot, LightKind::Directional] {
            *options.light_kind.clone().value_mut() = kind;
            harness.render();
        }

        // Spinning, with time moving on so the animation runs.
        *options.spin.clone().value_mut() = true;
        harness.render();
        harness.advance_time(std::time::Duration::from_secs(1));
        harness.render();
    }

    /// The cube on the floor, with the side bar's shadows toggle off and on. Pins down that the
    /// toggle reaches the views: turning it on changes many pixels, not just the checkbox.
    #[test]
    fn the_shadows_toggle_reaches_the_views() {
        let options = Options::new();
        *options.floor.clone().value_mut() = true;
        *options.shadows.clone().value_mut() = false;

        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();
        let off = harness.pixels().as_raw().to_vec();

        *options.shadows.clone().value_mut() = true;
        harness.render();
        let on = harness.pixels().as_raw().to_vec();

        let changed = off.iter().zip(&on).filter(|(a, b)| a != b).count();
        assert!(changed > 10_000, "only {changed} bytes changed: the shadow should show on the floor");
    }

    /// A material folder's maps are recognised by the words material sites put in their file
    /// names, in either naming style, and files that are not maps are left out.
    #[test]
    fn material_maps_are_recognised_by_name() {
        let folder = std::env::temp_dir().join(format!("carbide_playground_material_{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        for name in [
            "Metal012_1K-PNG_Color.png",
            "Metal012_1K-PNG_NormalGL.png",
            "Metal012_1K-PNG_Roughness.png",
            "Metal012_1K-PNG_Metalness.png",
            "Metal012_1K-PNG_AmbientOcclusion.png",
            "Metal012_1K-PNG_Emission.jpg",
            "Metal012_1K-PNG_Displacement.png",
            "Metal012.usdc",
        ] {
            std::fs::write(folder.join(name), b"").unwrap();
        }

        let surface = super::playground::material_folder(&folder).unwrap();
        std::fs::remove_dir_all(&folder).unwrap();

        let file = |id: &Option<carbide::draw::ImageId>| match id {
            Some(carbide::draw::ImageId::Local(path, _)) => path.file_name().unwrap().to_string_lossy().to_string(),
            other => panic!("not a local image: {other:?}"),
        };
        assert_eq!(file(&surface.albedo), "Metal012_1K-PNG_Color.png");
        assert_eq!(file(&surface.roughness), "Metal012_1K-PNG_Roughness.png");
        assert_eq!(file(&surface.metallic), "Metal012_1K-PNG_Metalness.png");
        assert_eq!(file(&surface.ambient_occlusion), "Metal012_1K-PNG_AmbientOcclusion.png");
        assert_eq!(file(&surface.emission), "Metal012_1K-PNG_Emission.jpg");
        let (normal, direction) = surface.normal.clone().unwrap();
        assert_eq!(file(&Some(normal)), "Metal012_1K-PNG_NormalGL.png");
        assert_eq!(direction, carbide::d3::material::normal_texture_y_direction::NormalTextureYDirection::Up);
    }

    /// The surfaces read at startup: the images, and a surface per folder in `assets/materials`,
    /// with its maps recognised by their file names.
    #[test]
    fn the_material_folders_become_surfaces() {
        let options = Options::new();
        let names: Vec<&str> = options.surfaces.iter().map(|surface| surface.name.as_str()).collect();
        assert_eq!(&names[..4], &["None", "Landscape", "Pixel grid", "Rust logo"]);

        for name in ["Ground068", "Plaster"] {
            let surface = options.surfaces.iter().find(|surface| surface.name == name).unwrap_or_else(|| panic!("no {name} in {names:?}"));
            assert!(surface.albedo.is_some(), "{name}: no colour map");
            assert!(surface.normal.is_some(), "{name}: no normal map");
            assert!(surface.roughness.is_some(), "{name}: no roughness map");
        }

        let ground = options.surfaces.iter().find(|surface| surface.name == "Ground068").unwrap();
        assert!(ground.ambient_occlusion.is_some());
    }

    /// The cube with each surface from the side bar, alone and times the vertex colours. Pins down
    /// that the picker reaches the material, and that the images load from `assets`: each surface
    /// changes many pixels from the one before it.
    #[test]
    fn each_surface_reaches_the_model() {
        let options = Options::new();
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();
        let mut previous = harness.pixels().as_raw().to_vec();

        for surface in options.surfaces.iter().skip(1) {
            *options.surface.clone().value_mut() = surface.clone();
            harness.render();
            let pixels = harness.pixels().as_raw().to_vec();

            let changed = previous.iter().zip(&pixels).filter(|(a, b)| a != b).count();
            assert!(changed > 10_000, "{}: only {changed} bytes changed", surface.name);
            previous = pixels;
        }

        *options.vertex_colors.clone().value_mut() = true;
        harness.render();
        *options.surface.clone().value_mut() = Surface::none();
        harness.render();
    }

    /// Each of a material folder's maps can be switched off and on from the side bar: off, the
    /// model changes; on again, it is as it was.
    #[test]
    fn each_map_can_be_switched_off() {
        let options = Options::new();
        *options.surface.clone().value_mut() = options.surfaces.iter().find(|surface| surface.name == "Ground068").unwrap().clone();
        // Rough enough that the roughness map shows, at its own values.
        *options.roughness.clone().value_mut() = 1.0;

        // The 3D view only: the side bar's checkboxes change too.
        let view = |harness: &mut Harness| -> Vec<u8> {
            let pixels = harness.pixels();
            (0..800).flat_map(|y| (0..880).map(move |x| (x, y))).flat_map(|(x, y)| pixels.get_pixel(x, y).0).collect()
        };

        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();
        let all_maps = view(&mut harness);

        for (name, toggle) in [
            ("colour", &options.use_color_map),
            ("normal", &options.use_normal_map),
            ("roughness", &options.use_roughness_map),
            ("occlusion", &options.use_occlusion_map),
        ] {
            *toggle.clone().value_mut() = false;
            harness.render();
            let without = view(&mut harness);
            let changed = all_maps.iter().zip(&without).filter(|(a, b)| a != b).count();
            assert!(changed > 1_000, "the {name} map off changes only {changed} bytes of the view");

            *toggle.clone().value_mut() = true;
            harness.render();
            assert!(view(&mut harness) == all_maps, "the {name} map on again doesn't restore the model");
        }
    }

    /// The tiling slider and the sampling picker reach the material: each changes the model, on a
    /// material folder's surface.
    #[test]
    fn tiling_and_sampling_reach_the_model() {
        let options = Options::new();
        *options.surface.clone().value_mut() = options.surfaces.iter().find(|surface| surface.name == "Plaster").unwrap().clone();

        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();
        let view = |harness: &mut Harness| -> Vec<u8> {
            let pixels = harness.pixels();
            (0..800).flat_map(|y| (0..880).map(move |x| (x, y))).flat_map(|(x, y)| pixels.get_pixel(x, y).0).collect()
        };
        let changed = |a: &[u8], b: &[u8]| a.iter().zip(b).filter(|(x, y)| x != y).count();
        let once = view(&mut harness);

        *options.tiling.clone().value_mut() = 4.0;
        harness.render();
        let tiled = view(&mut harness);
        assert!(changed(&once, &tiled) > 1_000, "tiling changes only {} bytes of the view", changed(&once, &tiled));

        *options.sampling.clone().value_mut() = carbide::d3::material::sample_type::SampleType::Nearest;
        harness.render();
        let nearest = view(&mut harness);
        assert!(changed(&tiled, &nearest) > 1_000, "nearest sampling changes only {} bytes of the view", changed(&tiled, &nearest));
    }

    /// The normal and occlusion strength sliders reach the model: with Ground068, which has both
    /// maps, flattening the normal map and switching off occlusion each change the view, and
    /// setting them back restores it.
    #[test]
    fn map_strengths_reach_the_model() {
        let options = Options::new();
        *options.surface.clone().value_mut() = options.surfaces.iter().find(|surface| surface.name == "Ground068").unwrap().clone();

        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();
        let view = |harness: &mut Harness| -> Vec<u8> {
            let pixels = harness.pixels();
            (0..800).flat_map(|y| (0..880).map(move |x| (x, y))).flat_map(|(x, y)| pixels.get_pixel(x, y).0).collect()
        };
        let changed = |a: &[u8], b: &[u8]| a.iter().zip(b).filter(|(x, y)| x != y).count();
        let full = view(&mut harness);

        *options.normal_scale.clone().value_mut() = 0.0;
        harness.render();
        let flat = view(&mut harness);
        assert!(changed(&full, &flat) > 1_000, "a normal strength of 0 changes only {} bytes of the view", changed(&full, &flat));

        *options.normal_scale.clone().value_mut() = 1.0;
        *options.occlusion_strength.clone().value_mut() = 0.0;
        harness.render();
        let unoccluded = view(&mut harness);
        assert!(changed(&full, &unoccluded) > 1_000, "an occlusion strength of 0 changes only {} bytes of the view", changed(&full, &unoccluded));

        *options.occlusion_strength.clone().value_mut() = 1.0;
        harness.render();
        assert_eq!(changed(&full, &view(&mut harness)), 0, "setting the strengths back doesn't restore the view");
    }

    /// Each OBJ model in the library's ressources loads with its MTL materials and draws. The
    /// wolf's material has its four maps: colour, normal (`map_Bump`), roughness and metallic.
    #[test]
    #[ignore = "slow: about 40 s in a debug build, parsing the large OBJ files; run with --ignored"]
    fn each_obj_model_loads_and_draws() {
        let wolf = Model::Wolf.load().unwrap();
        let (_, material) = &wolf.parts[0];
        assert!(material.albedo.value().to_texture().is_some(), "the wolf has no colour map");
        assert!(material.normal.value().to_texture().is_some(), "the wolf has no normal map");
        let maps = material.aomr_textures.value();
        assert!(maps.to_roughness_texture().is_some() && maps.to_metallic_texture().is_some(), "the wolf has no roughness or metallic map");

        let options = Options::new();
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        let view = |harness: &mut Harness| -> Vec<u8> {
            let pixels = harness.pixels();
            (0..800).flat_map(|y| (0..880).map(move |x| (x, y))).flat_map(|(x, y)| pixels.get_pixel(x, y).0).collect()
        };

        *options.model.clone().value_mut() = Model::Empty;
        harness.render();
        let empty = view(&mut harness);

        for model in [Model::RenderBall, Model::Wolf, Model::MoriKnob, Model::SportsCar] {
            *options.model.clone().value_mut() = model;
            harness.render();
            let drawn = view(&mut harness);
            let changed = empty.iter().zip(&drawn).filter(|(a, b)| a != b).count();
            assert!(changed > 10_000, "{model:?} changes only {changed} bytes of the empty view");
        }
    }

    /// The cube in the main view, hovered and then clicked twice. Pins down picking in the
    /// playground: the pointer over the cube sets the hovered state, a click selects it and records
    /// where it hit, and a second click deselects it.
    #[test]
    fn clicking_the_model_selects_it() {
        let options = Options::new();
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();

        // The middle of the main view, where the cube is.
        let centre = Position::new(450.0, 400.0);

        // Entering the view asks for a frame with a picking buffer; the update after it picks
        // hover from that buffer.
        harness.mouse_move(centre);
        harness.render();
        harness.render();
        assert!(*options.hovered.value(), "the pointer is over the cube");

        harness.click(centre);
        assert!(*options.outlined.value(), "a click selects the cube");
        assert!(options.last_hit.value().is_some(), "and records where it hit");

        harness.render();
        harness.click(centre);
        assert!(!*options.outlined.value(), "a second click deselects it");

        // Selected again, then a click beside the cube, on nothing: deselected.
        harness.render();
        harness.click(centre);
        assert!(*options.outlined.value());
        harness.render();
        harness.click(Position::new(100.0, 100.0));
        assert!(!*options.outlined.value(), "a click on nothing deselects it");

        // Still inside the view, off the cube: hover is picked again in the next frame.
        harness.mouse_move(Position::new(20.0, 20.0));
        harness.render();
        assert!(!*options.hovered.value(), "off the cube");
    }

    /// The red cube rendered with wgpu, then with the renderer set to None. Pins down that the
    /// side bar's renderer reaches the views: with none, they draw nothing, so the view's centre is
    /// no longer the cube's red; with wgpu again, the cube is back.
    #[test]
    fn choosing_no_renderer_draws_nothing() {
        let options = Options::new();
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();

        let red = |harness: &mut Harness| {
            let [r, g, b, _] = harness.pixels().get_pixel(450, 400).0;
            r > 120 && g < 60 && b < 60
        };
        assert!(red(&mut harness), "the cube, rendered with wgpu");

        *options.renderer.clone().value_mut() = Renderer::None;
        harness.render();
        harness.render();
        assert!(!red(&mut harness), "no renderer: the cube is gone");

        *options.renderer.clone().value_mut() = Renderer::Wgpu;
        harness.render();
        harness.render();
        assert!(red(&mut harness), "wgpu again: the cube is back");
    }

    /// The cube with the side bar's "Hover highlight" off, hovered and clicked. Pins down that the
    /// toggle removes hover: the hovered state stays false, while a click still selects the cube.
    #[test]
    fn turning_off_hover_keeps_clicks() {
        let options = Options::new();
        *options.hover.clone().value_mut() = false;
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();

        let centre = Position::new(450.0, 400.0);
        harness.mouse_move(centre);
        harness.render();
        harness.render();
        assert!(!*options.hovered.value(), "hover isn't tracked");

        harness.click(centre);
        assert!(*options.outlined.value(), "a click still selects the cube");
    }

    /// The cube with the side bar's "Interactive" off, hovered and clicked. Pins down that the
    /// toggle reaches the views: nothing is hovered or selected.
    #[test]
    fn turning_off_interactivity_ignores_clicks() {
        let options = Options::new();
        *options.interactive.clone().value_mut() = false;
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();

        let centre = Position::new(450.0, 400.0);
        harness.mouse_move(centre);
        harness.render();
        harness.render();
        harness.click(centre);

        assert!(!*options.hovered.value(), "nothing is hovered");
        assert!(!*options.outlined.value(), "nothing is selected");
    }

    /// The main view, orthographic, pinched outwards. Pins down that the pinch zooms by changing the
    /// camera's projection, which writes back into the side bar's height, so the slider follows.
    #[test]
    fn pinching_an_orthographic_view_moves_the_height_slider() {
        let options = Options::new();
        *options.projection.clone().value_mut() = ProjectionKind::Orthographic;

        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();

        harness.mouse_move(Position::new(450.0, 400.0));
        harness.pinch(0.5);
        harness.render();

        let height = *options.height.clone().value();
        assert!((height - 2.0 * (-0.5f64).exp()).abs() < 1e-3, "the height slider shows {height}");
    }
}

