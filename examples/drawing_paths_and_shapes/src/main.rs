use carbide::{Application, Window};
use carbide::draw::Dimension;

use crate::badge::Badge;

mod hexagon_parameters;
mod badge_background;
mod badge_symbol;
mod badge;

// This whole example is created by following the SwiftUI tutorial, but implemented with Carbide instead
// https://developer.apple.com/tutorials/swiftui/drawing-paths-and-shapes
fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Drawing paths and shapes",
        Dimension::new(300.0, 500.0),
        Badge::new()
    ));

    application.launch()
}
