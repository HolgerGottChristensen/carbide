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

    use super::playground::{playground, LightColor, LightKind, Model, Options, ProjectionKind, Renderer, Views};

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

        harness.mouse_move(centre);
        assert!(*options.hovered.value(), "the pointer is over the cube");

        harness.click(centre);
        assert!(*options.outlined.value(), "a click selects the cube");
        assert!(options.last_hit.value().is_some(), "and records where it hit");

        harness.render();
        harness.click(centre);
        assert!(!*options.outlined.value(), "a second click deselects it");

        harness.mouse_move(Position::new(20.0, 20.0));
        assert!(!*options.hovered.value(), "off the cube");
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

