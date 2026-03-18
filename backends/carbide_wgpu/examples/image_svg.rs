use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{AnyState, Map1, ReadStateExtNew, State};
use carbide_core::widget::*;
use carbide_icons::all_icon_names;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "Image SVG example - Carbide",
        Dimension::new(800.0, 600.0),
        //Image::new("images/ambulance.svg").resizeable()
        //Image::new("images/landscape.svg").resizeable()
        Image::new("images/lyon-logo.svg").resizeable()
        //Image::system("chart.network").resizeable()
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/circle/simple-case.svg")
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/ellipse/simple-case.svg")
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/line/simple-case.svg")
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/path/M-L.svg")
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/polygon/simple-case.svg")
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/polyline/simple-case.svg")
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/rect/simple-case.svg")
            .foreground_color(EnvironmentColor::Accent)
            .border()
            .color(EnvironmentColor::Label)
            .padding(100.0)
    ));

    application.launch();
}
