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

    use super::playground::{playground, LightKind, Model, Options, ProjectionKind, Renderer, Views};

    /// Every model, in one and two views, with both projections and both renderers, renders, and
    /// the main view can be dragged. A smoke test: it renders each combination without comparing
    /// pixels.
    #[test]
    fn every_option_renders() {
        let options = Options::new();
        let mut harness = Harness::new(playground(options.clone()), Dimension::new(1200.0, 800.0));
        harness.render();

        for model in [Model::Empty, Model::Cube, Model::Sphere, Model::Cubes, Model::RenderBall] {
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

